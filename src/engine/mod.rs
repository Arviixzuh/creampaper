//! The wallpaper engine runs on its own thread with its own message loop so
//! video playback never competes with the UI thread. The UI talks to it
//! through [`EngineHandle`] and receives [`EngineEvent`]s back on the UI
//! thread through `AppHandle::spawn_stream`.

use crate::model::Settings;
use creamui::render::AppHandle;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU32;
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;

#[cfg(not(windows))]
mod stub;
#[cfg(not(windows))]
use stub as platform;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
use self::windows as platform;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(not(windows), allow(dead_code))]
pub enum PauseReason {
    #[default]
    User,
    Fullscreen,
    Battery,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlaybackStatus {
    #[default]
    Idle,
    Playing,
    Paused(PauseReason),
}

#[cfg_attr(not(windows), allow(dead_code))]
pub enum EngineCommand {
    Configure(Settings),
    Play(PathBuf),
    Pause,
    Resume,
    Stop,
    Shutdown,
}

#[derive(Debug)]
#[cfg_attr(not(windows), allow(dead_code))]
pub enum EngineEvent {
    Status(PlaybackStatus),
    Error(String),
    /// The tray asked to bring the main window back.
    ShowWindow,
    /// The tray stopped the wallpaper.
    Stopped,
    /// The tray asked to exit the application.
    QuitRequested,
    /// The engine thread finished after a shutdown request.
    Exited,
}

pub struct Thumbnail {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Cheap to clone; every clone feeds the same engine thread.
#[derive(Clone)]
pub struct EngineHandle {
    commands: Sender<EngineCommand>,
    thread_id: Arc<AtomicU32>,
}

impl EngineHandle {
    pub fn send(&self, command: EngineCommand) {
        if self.commands.send(command).is_ok() {
            platform::wake(&self.thread_id);
        }
    }
}

/// Starts the engine thread. `on_event` always runs on the UI thread.
pub fn start(
    app: &AppHandle,
    settings: Settings,
    on_event: impl FnMut(EngineEvent) + 'static,
) -> EngineHandle {
    let (commands, receiver) = mpsc::channel();
    let thread_id = Arc::new(AtomicU32::new(0));
    let engine_thread_id = thread_id.clone();
    app.spawn_stream(
        move |emit| platform::run(receiver, settings, emit, engine_thread_id),
        on_event,
    );
    EngineHandle {
        commands,
        thread_id,
    }
}

/// Whether this build can actually draw behind the desktop icons.
pub const SUPPORTED: bool = cfg!(windows);

/// Explorer's cached thumbnail for a video, or `None` when unavailable.
/// Blocking: call it from a background thread.
pub fn thumbnail(path: &Path, width: u32, height: u32) -> Option<Thumbnail> {
    platform::thumbnail(path, width, height)
}

pub fn set_autostart(enabled: bool) -> Result<(), String> {
    platform::set_autostart(enabled)
}

/// Calls `on_restore` whenever the OS needs the whole window repainted
/// (restored from the taskbar, shown again, resized).
pub fn on_window_restored(window: &creamui::render::WindowHandle, on_restore: impl Fn() + 'static) {
    platform::on_window_restored(window, on_restore)
}

/// Returns `true` after asking an already running instance to show itself.
pub fn activate_existing_instance() -> bool {
    platform::activate_existing_instance()
}
