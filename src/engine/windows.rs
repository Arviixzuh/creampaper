//! Windows backend.
//!
//! - Video is decoded by Media Foundation (MFPlay), which uses DXVA hardware
//!   decoding and the EVR presenter: no browser, no custom renderer.
//! - One borderless child window per monitor is parented to the desktop's
//!   `WorkerW` layer, so the video sits behind the icons.
//! - The thread sleeps in `GetMessageW`; the only periodic work is a 1.5 s
//!   timer that checks for fullscreen/maximized windows and battery.
//! - The tray icon lives on this thread too, so the UI window can be fully
//!   hidden while the wallpaper keeps playing.

use super::{EngineCommand, EngineEvent, PauseReason, PlaybackStatus, Thumbnail};
use crate::model::{FitMode, MonitorMode, Settings};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use windows::core::{implement, w, PCWSTR, PROPVARIANT};
use windows::Win32::Foundation::{
    GetLastError, BOOL, ERROR_ALREADY_EXISTS, ERROR_SUCCESS, HWND, LPARAM, LRESULT, POINT, RECT,
    SIZE, WPARAM,
};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::Graphics::Gdi::{
    CreateSolidBrush, DeleteObject, EnumDisplayMonitors, GetDC, GetDIBits, GetMonitorInfoW,
    GetObjectW, MapWindowPoints, MonitorFromWindow, ReleaseDC, BITMAP, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP, HDC, HMONITOR, MONITORINFO,
    MONITOR_DEFAULTTONULL,
};
use windows::Win32::Media::MediaFoundation::{
    IMFPMediaPlayer, IMFPMediaPlayerCallback, IMFPMediaPlayerCallback_Impl, MFPCreateMediaPlayer,
    MFShutdown, MFStartup, MFVideoARMode_None, MFVideoARMode_PreservePicture,
    MFVideoNormalizedRect, MFP_EVENT_HEADER, MFP_EVENT_TYPE_ERROR, MFP_EVENT_TYPE_MEDIAITEM_SET,
    MFP_EVENT_TYPE_PLAYBACK_ENDED, MFP_OPTION_NONE, MFP_POSITIONTYPE_100NS, MFSTARTUP_LITE,
    MF_VERSION,
};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
use windows::Win32::System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleW};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows::Win32::System::Registry::{
    RegDeleteKeyValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ,
};
use windows::Win32::System::Threading::{CreateMutexW, GetCurrentThreadId};
use windows::Win32::UI::Shell::{
    DefSubclassProc, IShellItemImageFactory, SHCreateItemFromParsingName,
    SHQueryUserNotificationState, SetWindowSubclass, Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE,
    NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, QUNS_BUSY, QUNS_PRESENTATION_MODE,
    QUNS_RUNNING_D3D_FULL_SCREEN, SIIGBF_BIGGERSIZEOK, SIIGBF_THUMBNAILONLY,
};
use windows::Win32::UI::WindowsAndMessaging::*;

const WM_WAKE: u32 = WM_APP + 1;
const WM_TRAY: u32 = WM_APP + 2;
const WM_ACTION: u32 = WM_APP + 3;
const WM_MEDIA_READY: u32 = WM_APP + 4;
const WM_MEDIA_ERROR: u32 = WM_APP + 5;
const WM_RELAYOUT: u32 = WM_APP + 6;
const WM_SHOW_EXISTING: u32 = WM_APP + 7;

const ACTION_SHOW: usize = 1;
const ACTION_TOGGLE: usize = 2;
const ACTION_STOP: usize = 3;
const ACTION_QUIT: usize = 4;

const POLICY_TIMER: usize = 1;
const POLICY_INTERVAL_MS: u32 = 1500;

const HOST_CLASS: PCWSTR = w!("CreamPaperHost");
const TRAY_CLASS: PCWSTR = w!("CreamPaperTray");
const INSTANCE_MUTEX: PCWSTR = w!("Local\\CreamPaper.Instance");
const RUN_KEY: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const RUN_VALUE: PCWSTR = w!("CreamPaper");

static ENGINE_THREAD: AtomicU32 = AtomicU32::new(0);

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn post_to_engine(message: u32, wparam: usize) {
    let thread = ENGINE_THREAD.load(Ordering::Acquire);
    if thread != 0 {
        unsafe {
            let _ = PostThreadMessageW(thread, message, WPARAM(wparam), LPARAM(0));
        }
    }
}

pub fn wake(thread_id: &AtomicU32) {
    let thread = thread_id.load(Ordering::Acquire);
    if thread != 0 {
        unsafe {
            let _ = PostThreadMessageW(thread, WM_WAKE, WPARAM(0), LPARAM(0));
        }
    }
}

// ---------------------------------------------------------------------------
// MFPlay callback: forwards interesting events to the engine loop.

#[implement(IMFPMediaPlayerCallback)]
struct PlayerEvents;

impl IMFPMediaPlayerCallback_Impl for PlayerEvents_Impl {
    fn OnMediaPlayerEvent(&self, header: *const MFP_EVENT_HEADER) {
        let Some(header) = (unsafe { header.as_ref() }) else {
            return;
        };
        match header.eEventType {
            MFP_EVENT_TYPE_PLAYBACK_ENDED => {
                // Seamless loop: rewind and keep going.
                if let Some(player) = header.pMediaPlayer.as_ref() {
                    unsafe {
                        let start = PROPVARIANT::from(0i64);
                        let _ = player.SetPosition(&MFP_POSITIONTYPE_100NS, &start);
                        let _ = player.Play();
                    }
                }
            }
            MFP_EVENT_TYPE_MEDIAITEM_SET => post_to_engine(WM_MEDIA_READY, 0),
            MFP_EVENT_TYPE_ERROR => {
                post_to_engine(WM_MEDIA_ERROR, header.hrEvent.0 as u32 as usize)
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Desktop layer discovery.

struct DesktopLayer {
    parent: HWND,
}

unsafe extern "system" fn find_workerw(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let found = &mut *(lparam.0 as *mut HWND);
    if FindWindowExW(hwnd, None, w!("SHELLDLL_DefView"), None).is_ok() {
        if let Ok(worker) = FindWindowExW(None, hwnd, w!("WorkerW"), None) {
            *found = worker;
            return BOOL(0);
        }
    }
    BOOL(1)
}

/// Asks Explorer to spawn the `WorkerW` that sits between the wallpaper and
/// the icons, then returns the window video hosts should be parented to.
fn desktop_layer() -> Option<DesktopLayer> {
    unsafe {
        let progman = FindWindowW(w!("Progman"), None).ok()?;
        let mut result = 0usize;
        SendMessageTimeoutW(
            progman,
            0x052C,
            WPARAM(0xD),
            LPARAM(0x1),
            SMTO_NORMAL,
            1000,
            Some(&mut result),
        );
        SendMessageTimeoutW(
            progman,
            0x052C,
            WPARAM(0),
            LPARAM(0),
            SMTO_NORMAL,
            1000,
            Some(&mut result),
        );

        let mut worker = HWND::default();
        let _ = EnumWindows(
            Some(find_workerw),
            LPARAM(&mut worker as *mut HWND as isize),
        );
        if !worker.is_invalid() {
            return Some(DesktopLayer { parent: worker });
        }
        // Windows 11 24H2+: the WorkerW is a child of Progman; hosts go into
        // Progman right below the icon view.
        Some(DesktopLayer { parent: progman })
    }
}

unsafe fn icon_view(parent: HWND) -> Option<HWND> {
    FindWindowExW(parent, None, w!("SHELLDLL_DefView"), None).ok()
}

// ---------------------------------------------------------------------------
// Monitors.

#[derive(Clone, Copy)]
struct Monitor {
    handle: HMONITOR,
    rect: RECT,
    primary: bool,
}

unsafe extern "system" fn collect_monitor(
    handle: HMONITOR,
    _: HDC,
    _: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let monitors = &mut *(lparam.0 as *mut Vec<Monitor>);
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if GetMonitorInfoW(handle, &mut info).as_bool() {
        monitors.push(Monitor {
            handle,
            rect: info.rcMonitor,
            primary: info.dwFlags & MONITORINFOF_PRIMARY != 0,
        });
    }
    BOOL(1)
}

fn monitors(mode: MonitorMode) -> Vec<Monitor> {
    let mut list: Vec<Monitor> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(collect_monitor),
            LPARAM(&mut list as *mut _ as isize),
        );
    }
    if mode == MonitorMode::Primary {
        list.retain(|monitor| monitor.primary);
    }
    list.sort_by_key(|monitor| !monitor.primary);
    list
}

// ---------------------------------------------------------------------------
// One video surface per monitor.

struct Surface {
    monitor: Monitor,
    host: HWND,
    player: IMFPMediaPlayer,
    auto_paused: bool,
}

impl Surface {
    fn create(
        layer: &DesktopLayer,
        monitor: Monitor,
        path: &Path,
        events: &IMFPMediaPlayerCallback,
    ) -> Result<Self, String> {
        unsafe {
            let mut corners = [
                POINT {
                    x: monitor.rect.left,
                    y: monitor.rect.top,
                },
                POINT {
                    x: monitor.rect.right,
                    y: monitor.rect.bottom,
                },
            ];
            MapWindowPoints(None, layer.parent, &mut corners);
            let host = CreateWindowExW(
                WS_EX_NOACTIVATE,
                HOST_CLASS,
                w!(""),
                WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS,
                corners[0].x,
                corners[0].y,
                corners[1].x - corners[0].x,
                corners[1].y - corners[0].y,
                layer.parent,
                None,
                windows::Win32::Foundation::HINSTANCE::from(
                    GetModuleHandleW(None).unwrap_or_default(),
                ),
                None,
            )
            .map_err(|error| format!("No se pudo crear la superficie de video: {error}"))?;
            if let Some(view) = icon_view(layer.parent) {
                let _ = SetWindowPos(
                    host,
                    view,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }

            let url = wide(&path.to_string_lossy());
            let mut player = None;
            let created = MFPCreateMediaPlayer(
                PCWSTR(url.as_ptr()),
                BOOL(1),
                MFP_OPTION_NONE,
                events,
                host,
                Some(&mut player),
            );
            match (created, player) {
                (Ok(()), Some(player)) => Ok(Surface {
                    monitor,
                    host,
                    player,
                    auto_paused: false,
                }),
                (result, _) => {
                    let _ = DestroyWindow(host);
                    let detail = result
                        .err()
                        .map(|error| error.message())
                        .unwrap_or_default();
                    Err(format!("Media Foundation no pudo abrir el video. {detail}"))
                }
            }
        }
    }

    fn apply(&self, settings: &Settings, audible: bool) {
        unsafe {
            let _ = self.player.SetMute(BOOL::from(settings.muted || !audible));
            let _ = self.player.SetVolume(settings.volume.clamp(0.0, 1.0));
            let full = MFVideoNormalizedRect {
                left: 0.0,
                top: 0.0,
                right: 1.0,
                bottom: 1.0,
            };
            match settings.fit {
                FitMode::Contain => {
                    let _ = self.player.SetVideoSourceRect(&full);
                    let _ = self
                        .player
                        .SetAspectRatioMode(MFVideoARMode_PreservePicture.0 as u32);
                }
                FitMode::Stretch => {
                    let _ = self.player.SetVideoSourceRect(&full);
                    let _ = self.player.SetAspectRatioMode(MFVideoARMode_None.0 as u32);
                }
                FitMode::Cover => {
                    let mut native = SIZE::default();
                    let mut ratio = SIZE::default();
                    if self
                        .player
                        .GetNativeVideoSize(Some(&mut native), Some(&mut ratio))
                        .is_ok()
                    {
                        let (w, h) = if ratio.cx > 0 && ratio.cy > 0 {
                            (ratio.cx, ratio.cy)
                        } else {
                            (native.cx, native.cy)
                        };
                        let screen_w =
                            (self.monitor.rect.right - self.monitor.rect.left).max(1) as f32;
                        let screen_h =
                            (self.monitor.rect.bottom - self.monitor.rect.top).max(1) as f32;
                        let _ = self.player.SetVideoSourceRect(&cover_rect(
                            w as f32,
                            h as f32,
                            screen_w / screen_h,
                        ));
                    }
                    let _ = self.player.SetAspectRatioMode(MFVideoARMode_None.0 as u32);
                }
            }
            let _ = self.player.UpdateVideo();
        }
    }

    fn set_paused(&self, paused: bool) {
        unsafe {
            let _ = if paused {
                self.player.Pause()
            } else {
                self.player.Play()
            };
        }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            let _ = self.player.Shutdown();
            let _ = DestroyWindow(self.host);
        }
    }
}

/// Normalized source crop that fills a screen of `screen_aspect` without bars.
fn cover_rect(video_w: f32, video_h: f32, screen_aspect: f32) -> MFVideoNormalizedRect {
    if video_w <= 0.0 || video_h <= 0.0 {
        return MFVideoNormalizedRect {
            left: 0.0,
            top: 0.0,
            right: 1.0,
            bottom: 1.0,
        };
    }
    let video_aspect = video_w / video_h;
    if video_aspect > screen_aspect {
        let visible = screen_aspect / video_aspect;
        let margin = (1.0 - visible) / 2.0;
        MFVideoNormalizedRect {
            left: margin,
            top: 0.0,
            right: 1.0 - margin,
            bottom: 1.0,
        }
    } else {
        let visible = video_aspect / screen_aspect;
        let margin = (1.0 - visible) / 2.0;
        MFVideoNormalizedRect {
            left: 0.0,
            top: margin,
            right: 1.0,
            bottom: 1.0 - margin,
        }
    }
}

// ---------------------------------------------------------------------------
// Window procedures.

unsafe extern "system" fn host_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

static TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);

unsafe extern "system" fn tray_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_TRAY => {
            match (lparam.0 as u32) & 0xFFFF {
                WM_LBUTTONUP | WM_LBUTTONDBLCLK => post_to_engine(WM_ACTION, ACTION_SHOW),
                WM_RBUTTONUP | WM_CONTEXTMENU => show_tray_menu(hwnd),
                _ => {}
            }
            LRESULT(0)
        }
        WM_SHOW_EXISTING => {
            post_to_engine(WM_ACTION, ACTION_SHOW);
            LRESULT(0)
        }
        WM_DISPLAYCHANGE | WM_SETTINGCHANGE => {
            post_to_engine(WM_RELAYOUT, 0);
            DefWindowProcW(hwnd, message, wparam, lparam)
        }
        WM_TIMER => {
            post_to_engine(WM_TIMER, wparam.0);
            LRESULT(0)
        }
        _ if message != 0 && message == TASKBAR_CREATED.load(Ordering::Relaxed) => {
            // Explorer restarted: the tray icon and the WorkerW are gone.
            post_to_engine(WM_RELAYOUT, 1);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

static TRAY_PLAYING: AtomicU32 = AtomicU32::new(0);

unsafe fn show_tray_menu(hwnd: HWND) {
    let Ok(menu) = CreatePopupMenu() else {
        return;
    };
    let toggle = if TRAY_PLAYING.load(Ordering::Relaxed) == 1 {
        w!("Pausar")
    } else {
        w!("Reanudar")
    };
    let _ = AppendMenuW(menu, MF_STRING, ACTION_SHOW, w!("Abrir CreamPaper"));
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
    let _ = AppendMenuW(menu, MF_STRING, ACTION_TOGGLE, toggle);
    let _ = AppendMenuW(menu, MF_STRING, ACTION_STOP, w!("Quitar fondo animado"));
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
    let _ = AppendMenuW(menu, MF_STRING, ACTION_QUIT, w!("Salir"));
    let mut cursor = POINT::default();
    let _ = GetCursorPos(&mut cursor);
    let _ = SetForegroundWindow(hwnd);
    let command = TrackPopupMenu(
        menu,
        TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_NONOTIFY,
        cursor.x,
        cursor.y,
        0,
        hwnd,
        None,
    );
    let _ = DestroyMenu(menu);
    if command.0 != 0 {
        post_to_engine(WM_ACTION, command.0 as usize);
    }
}

fn register_classes() {
    unsafe {
        let instance = GetModuleHandleW(None).unwrap_or_default();
        let host = WNDCLASSW {
            lpfnWndProc: Some(host_proc),
            hInstance: instance.into(),
            lpszClassName: HOST_CLASS,
            hbrBackground: CreateSolidBrush(windows::Win32::Foundation::COLORREF(0)),
            ..Default::default()
        };
        RegisterClassW(&host);
        let tray = WNDCLASSW {
            lpfnWndProc: Some(tray_proc),
            hInstance: instance.into(),
            lpszClassName: TRAY_CLASS,
            ..Default::default()
        };
        RegisterClassW(&tray);
        TASKBAR_CREATED.store(
            RegisterWindowMessageW(w!("TaskbarCreated")),
            Ordering::Relaxed,
        );
    }
}

fn tray_icon_data(hwnd: HWND) -> NOTIFYICONDATAW {
    let mut data = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: 1,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: WM_TRAY,
        ..Default::default()
    };
    unsafe {
        let instance = GetModuleHandleW(None).unwrap_or_default();
        data.hIcon = LoadIconW(instance, PCWSTR(1 as _))
            .or_else(|_| LoadIconW(None, IDI_APPLICATION))
            .unwrap_or_default();
    }
    for (slot, unit) in data.szTip.iter_mut().zip("CreamPaper".encode_utf16()) {
        *slot = unit;
    }
    data
}

// ---------------------------------------------------------------------------
// Auto-pause policy.

#[derive(Default)]
struct Occlusion {
    fullscreen: Vec<HMONITOR>,
    maximized: Vec<HMONITOR>,
}

unsafe fn is_cloaked(hwnd: HWND) -> bool {
    let mut cloaked = 0u32;
    DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &mut cloaked as *mut u32 as *mut _, 4).is_ok()
        && cloaked != 0
}

unsafe fn is_desktop_window(hwnd: HWND) -> bool {
    let mut class = [0u16; 32];
    let len = GetClassNameW(hwnd, &mut class) as usize;
    let name = String::from_utf16_lossy(&class[..len]);
    matches!(
        name.as_str(),
        "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd"
    )
}

unsafe extern "system" fn collect_maximized(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let found = &mut *(lparam.0 as *mut Vec<HMONITOR>);
    let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
    if IsWindowVisible(hwnd).as_bool()
        && IsZoomed(hwnd).as_bool()
        && !IsIconic(hwnd).as_bool()
        && ex_style & WS_EX_TOOLWINDOW.0 == 0
        && !is_cloaked(hwnd)
    {
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONULL);
        if !monitor.is_invalid() && !found.contains(&monitor) {
            found.push(monitor);
        }
    }
    BOOL(1)
}

fn occlusion(settings: &Settings) -> Occlusion {
    let mut result = Occlusion::default();
    unsafe {
        if settings.pause_on_fullscreen {
            let state = SHQueryUserNotificationState().unwrap_or_default();
            let busy = state == QUNS_BUSY
                || state == QUNS_RUNNING_D3D_FULL_SCREEN
                || state == QUNS_PRESENTATION_MODE;
            let foreground = GetForegroundWindow();
            if !foreground.is_invalid() && !is_desktop_window(foreground) {
                let monitor = MonitorFromWindow(foreground, MONITOR_DEFAULTTONULL);
                let mut info = MONITORINFO {
                    cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                    ..Default::default()
                };
                let mut rect = RECT::default();
                let covers = GetMonitorInfoW(monitor, &mut info).as_bool()
                    && GetWindowRect(foreground, &mut rect).is_ok()
                    && rect.left <= info.rcMonitor.left
                    && rect.top <= info.rcMonitor.top
                    && rect.right >= info.rcMonitor.right
                    && rect.bottom >= info.rcMonitor.bottom;
                if covers || busy {
                    result.fullscreen.push(monitor);
                }
            }
        }
        if settings.pause_on_maximized {
            let _ = EnumWindows(
                Some(collect_maximized),
                LPARAM(&mut result.maximized as *mut _ as isize),
            );
        }
    }
    result
}

fn on_battery() -> bool {
    let mut status = SYSTEM_POWER_STATUS::default();
    unsafe { GetSystemPowerStatus(&mut status).is_ok() && status.ACLineStatus == 0 }
}

// ---------------------------------------------------------------------------
// Static wallpaper restore: hosts leave their last frame on the WorkerW.

fn refresh_static_wallpaper() {
    unsafe {
        let mut path = [0u16; 520];
        if SystemParametersInfoW(
            SPI_GETDESKWALLPAPER,
            path.len() as u32,
            Some(path.as_mut_ptr() as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
        .is_ok()
        {
            let _ = SystemParametersInfoW(
                SPI_SETDESKWALLPAPER,
                0,
                Some(path.as_mut_ptr() as *mut _),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Engine loop.

struct Engine {
    settings: Settings,
    emit: Box<dyn Fn(EngineEvent) + Send>,
    events: IMFPMediaPlayerCallback,
    tray: HWND,
    path: Option<PathBuf>,
    surfaces: Vec<Surface>,
    user_paused: bool,
    battery_paused: bool,
    status: PlaybackStatus,
}

impl Engine {
    fn play(&mut self, path: PathBuf) {
        self.surfaces.clear();
        self.path = Some(path);
        self.user_paused = false;
        self.build_surfaces();
    }

    fn build_surfaces(&mut self) {
        self.surfaces.clear();
        let Some(path) = self.path.clone() else {
            return;
        };
        let Some(layer) = desktop_layer() else {
            (self.emit)(EngineEvent::Error(
                "No se encontró la capa del escritorio de Windows".into(),
            ));
            return;
        };
        for monitor in monitors(self.settings.monitors) {
            match Surface::create(&layer, monitor, &path, &self.events) {
                Ok(surface) => self.surfaces.push(surface),
                Err(error) => {
                    (self.emit)(EngineEvent::Error(error));
                    break;
                }
            }
        }
        if self.surfaces.is_empty() {
            self.path = None;
            (self.emit)(EngineEvent::Stopped);
        }
        self.apply_settings();
        self.evaluate();
    }

    fn stop(&mut self) {
        let had_video = !self.surfaces.is_empty();
        self.surfaces.clear();
        self.path = None;
        self.user_paused = false;
        if had_video {
            refresh_static_wallpaper();
        }
        self.evaluate();
    }

    fn apply_settings(&self) {
        for (index, surface) in self.surfaces.iter().enumerate() {
            surface.apply(&self.settings, index == 0);
        }
    }

    fn configure(&mut self, settings: Settings) {
        let relayout = settings.monitors != self.settings.monitors;
        self.settings = settings;
        if relayout && self.path.is_some() {
            self.build_surfaces();
        } else {
            self.apply_settings();
            self.evaluate();
        }
    }

    /// Recomputes which surfaces should run and reports the overall status.
    fn evaluate(&mut self) {
        self.battery_paused = self.settings.pause_on_battery && on_battery();
        let occluded = if self.surfaces.is_empty() {
            Occlusion::default()
        } else {
            occlusion(&self.settings)
        };
        let mut any_playing = false;
        let mut any_fullscreen = false;
        for surface in &mut self.surfaces {
            let fullscreen = occluded.fullscreen.contains(&surface.monitor.handle);
            let covered = fullscreen || occluded.maximized.contains(&surface.monitor.handle);
            any_fullscreen |= covered;
            let paused = self.user_paused || self.battery_paused || covered;
            if paused != surface.auto_paused {
                surface.set_paused(paused);
                surface.auto_paused = paused;
            }
            any_playing |= !paused;
        }
        let status = if self.surfaces.is_empty() {
            PlaybackStatus::Idle
        } else if any_playing {
            PlaybackStatus::Playing
        } else if self.user_paused {
            PlaybackStatus::Paused(PauseReason::User)
        } else if self.battery_paused {
            PlaybackStatus::Paused(PauseReason::Battery)
        } else if any_fullscreen {
            PlaybackStatus::Paused(PauseReason::Fullscreen)
        } else {
            PlaybackStatus::Paused(PauseReason::User)
        };
        TRAY_PLAYING.store(
            u32::from(status == PlaybackStatus::Playing),
            Ordering::Relaxed,
        );
        if status != self.status {
            self.status = status;
            (self.emit)(EngineEvent::Status(status));
        }
    }

    fn handle(&mut self, command: EngineCommand) -> bool {
        match command {
            EngineCommand::Configure(settings) => self.configure(settings),
            EngineCommand::Play(path) => self.play(path),
            EngineCommand::Pause => {
                self.user_paused = true;
                self.evaluate();
            }
            EngineCommand::Resume => {
                self.user_paused = false;
                self.evaluate();
            }
            EngineCommand::Stop => self.stop(),
            EngineCommand::Shutdown => return false,
        }
        true
    }

    fn action(&mut self, action: usize) {
        match action {
            ACTION_SHOW => (self.emit)(EngineEvent::ShowWindow),
            ACTION_TOGGLE if !self.surfaces.is_empty() => {
                self.user_paused = self.status == PlaybackStatus::Playing;
                self.evaluate();
            }
            ACTION_STOP => {
                self.stop();
                (self.emit)(EngineEvent::Stopped);
            }
            ACTION_QUIT => (self.emit)(EngineEvent::QuitRequested),
            _ => {}
        }
    }
}

pub fn run(
    commands: Receiver<EngineCommand>,
    settings: Settings,
    emit: Box<dyn Fn(EngineEvent) + Send>,
    thread_id: Arc<AtomicU32>,
) {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let media = MFStartup(MF_VERSION, MFSTARTUP_LITE);
        if let Err(error) = &media {
            emit(EngineEvent::Error(format!(
                "Media Foundation no está disponible: {error}"
            )));
        }
        // Ensure this thread has a message queue before publishing its id.
        let mut message = MSG::default();
        let _ = PeekMessageW(&mut message, None, WM_USER, WM_USER, PM_NOREMOVE);
        let current = GetCurrentThreadId();
        ENGINE_THREAD.store(current, Ordering::Release);
        thread_id.store(current, Ordering::Release);

        register_classes();
        let instance = GetModuleHandleW(None).unwrap_or_default();
        let tray = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            TRAY_CLASS,
            w!("CreamPaper"),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            None,
            None,
            instance,
            None,
        )
        .unwrap_or_default();
        let tray_data = tray_icon_data(tray);
        let _ = Shell_NotifyIconW(NIM_ADD, &tray_data);
        SetTimer(tray, POLICY_TIMER, POLICY_INTERVAL_MS, None);

        let mut engine = Engine {
            settings,
            emit,
            events: PlayerEvents.into(),
            tray,
            path: None,
            surfaces: Vec::new(),
            user_paused: false,
            battery_paused: false,
            status: PlaybackStatus::Idle,
        };

        let mut running = true;
        while running && GetMessageW(&mut message, None, 0, 0).as_bool() {
            if message.hwnd.is_invalid() {
                match message.message {
                    WM_ACTION => engine.action(message.wParam.0),
                    WM_MEDIA_READY => engine.apply_settings(),
                    WM_MEDIA_ERROR => {
                        let code = windows::core::HRESULT(message.wParam.0 as u32 as i32);
                        (engine.emit)(EngineEvent::Error(format!(
                            "Error al reproducir el video: {}",
                            code.message()
                        )));
                        engine.stop();
                        (engine.emit)(EngineEvent::Stopped);
                    }
                    WM_RELAYOUT => {
                        if message.wParam.0 == 1 {
                            let _ = Shell_NotifyIconW(NIM_ADD, &tray_data);
                        }
                        if engine.path.is_some() {
                            engine.build_surfaces();
                        }
                    }
                    WM_TIMER => engine.evaluate(),
                    _ => {}
                }
            } else {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            // Commands are drained on every wake-up, so a wake message lost
            // inside a modal loop (e.g. the tray menu) is harmless.
            while let Ok(command) = commands.try_recv() {
                if !engine.handle(command) {
                    running = false;
                    break;
                }
            }
        }

        engine.stop();
        let _ = KillTimer(engine.tray, POLICY_TIMER);
        let _ = Shell_NotifyIconW(NIM_DELETE, &tray_data);
        let _ = DestroyWindow(engine.tray);
        let emit = std::mem::replace(&mut engine.emit, Box::new(|_| {}));
        drop(engine);
        if media.is_ok() {
            let _ = MFShutdown();
        }
        CoUninitialize();
        ENGINE_THREAD.store(0, Ordering::Release);
        emit(EngineEvent::Exited);
    }
}

// ---------------------------------------------------------------------------
// Thumbnails from Explorer's cache (same images File Explorer shows).

pub fn thumbnail(path: &Path, width: u32, height: u32) -> Option<Thumbnail> {
    unsafe {
        let _ = CoInitializeEx(None, windows::Win32::System::Com::COINIT_MULTITHREADED);
        let result = shell_thumbnail(path, width, height);
        CoUninitialize();
        result
    }
}

unsafe fn shell_thumbnail(path: &Path, width: u32, height: u32) -> Option<Thumbnail> {
    let wide_path = wide(&path.to_string_lossy());
    let factory: IShellItemImageFactory =
        SHCreateItemFromParsingName(PCWSTR(wide_path.as_ptr()), None).ok()?;
    let size = SIZE {
        cx: width as i32,
        cy: height as i32,
    };
    let bitmap = factory
        .GetImage(size, SIIGBF_THUMBNAILONLY | SIIGBF_BIGGERSIZEOK)
        .ok()?;
    let pixels = bitmap_pixels(bitmap);
    let _ = DeleteObject(bitmap);
    pixels
}

unsafe fn bitmap_pixels(bitmap: HBITMAP) -> Option<Thumbnail> {
    let mut info = BITMAP::default();
    if GetObjectW(
        bitmap,
        std::mem::size_of::<BITMAP>() as i32,
        Some(&mut info as *mut _ as *mut _),
    ) == 0
    {
        return None;
    }
    let (width, height) = (
        info.bmWidth.max(1) as u32,
        info.bmHeight.unsigned_abs().max(1),
    );
    let mut header = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: -(height as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut buffer = vec![0u8; (width * height * 4) as usize];
    let dc = GetDC(None);
    let lines = GetDIBits(
        dc,
        bitmap,
        0,
        height,
        Some(buffer.as_mut_ptr() as *mut _),
        &mut header,
        DIB_RGB_COLORS,
    );
    ReleaseDC(None, dc);
    if lines == 0 {
        return None;
    }
    let opaque = buffer.as_chunks::<4>().0.iter().all(|pixel| pixel[3] == 0);
    for pixel in buffer.as_chunks_mut::<4>().0 {
        pixel.swap(0, 2);
        if opaque {
            pixel[3] = 255;
        }
    }
    Some(Thumbnail {
        width,
        height,
        rgba: buffer,
    })
}

// ---------------------------------------------------------------------------
// Start with Windows (per-user Run key, no admin rights needed).

pub fn set_autostart(enabled: bool) -> Result<(), String> {
    unsafe {
        let status = if enabled {
            let mut exe = [0u16; 1024];
            let len = GetModuleFileNameW(None, &mut exe) as usize;
            if len == 0 {
                return Err("No se pudo localizar el ejecutable".into());
            }
            let command = format!("\"{}\" --background", String::from_utf16_lossy(&exe[..len]));
            let value = wide(&command);
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                RUN_KEY,
                RUN_VALUE,
                REG_SZ.0,
                Some(value.as_ptr() as *const _),
                (value.len() * 2) as u32,
            )
        } else {
            let status = RegDeleteKeyValueW(HKEY_CURRENT_USER, RUN_KEY, RUN_VALUE);
            if status == windows::Win32::Foundation::ERROR_FILE_NOT_FOUND {
                ERROR_SUCCESS
            } else {
                status
            }
        };
        if status == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(format!(
                "No se pudo actualizar el inicio automático (código {})",
                status.0
            ))
        }
    }
}

// ---------------------------------------------------------------------------
// Main window: full repaint after restore.

const WM_FULL_REPAINT: u32 = WM_APP + 20;
const REPAINT_SUBCLASS: usize = 0xC9EA;

thread_local! {
    static ON_RESTORE: std::cell::RefCell<Option<Box<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
}

unsafe extern "system" fn find_main_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let mut title = [0u16; 32];
    let len = GetWindowTextW(hwnd, &mut title) as usize;
    if String::from_utf16_lossy(&title[..len]) == "CreamPaper" {
        *(lparam.0 as *mut HWND) = hwnd;
        return BOOL(0);
    }
    BOOL(1)
}

unsafe extern "system" fn repaint_subclass(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _: usize,
    _: usize,
) -> LRESULT {
    match message {
        // These can arrive synchronously from inside CreamUI calls
        // (e.g. `set_minimized`), so the repaint is deferred to the queue.
        WM_SIZE if wparam.0 as u32 != SIZE_MINIMIZED => {
            let _ = PostMessageW(hwnd, WM_FULL_REPAINT, WPARAM(0), LPARAM(0));
        }
        WM_SHOWWINDOW if wparam.0 != 0 => {
            let _ = PostMessageW(hwnd, WM_FULL_REPAINT, WPARAM(0), LPARAM(0));
        }
        WM_FULL_REPAINT => {
            ON_RESTORE.with(|hook| {
                if let Some(hook) = hook.borrow().as_ref() {
                    hook();
                }
            });
            return LRESULT(0);
        }
        _ => {}
    }
    DefSubclassProc(hwnd, message, wparam, lparam)
}

/// Must run on the UI thread, which owns the CreamUI window.
pub fn on_window_restored(
    _window: &creamui::render::WindowHandle,
    on_restore: impl Fn() + 'static,
) {
    unsafe {
        let mut hwnd = HWND::default();
        let _ = EnumThreadWindows(
            GetCurrentThreadId(),
            Some(find_main_window),
            LPARAM(&mut hwnd as *mut HWND as isize),
        );
        if hwnd.is_invalid() {
            return;
        }
        ON_RESTORE.with(|hook| *hook.borrow_mut() = Some(Box::new(on_restore)));
        let _ = SetWindowSubclass(hwnd, Some(repaint_subclass), REPAINT_SUBCLASS, 0);
    }
}

// ---------------------------------------------------------------------------
// Single instance: a second launch just brings the first one forward.

pub fn activate_existing_instance() -> bool {
    unsafe {
        let mutex = CreateMutexW(None, BOOL(1), INSTANCE_MUTEX);
        if mutex.is_ok() && GetLastError() != ERROR_ALREADY_EXISTS {
            // Held for the lifetime of the process on purpose.
            std::mem::forget(mutex);
            return false;
        }
        if let Ok(existing) = FindWindowW(TRAY_CLASS, None) {
            let _ = PostMessageW(existing, WM_SHOW_EXISTING, WPARAM(0), LPARAM(0));
        }
        true
    }
}
