//! Application wiring: window creation, engine start-up and lifecycle.

mod state;

pub use state::{AppState, Notice, NoticeKind, Page};

use crate::{engine, model, ui};
use creamui::render::{AppBuilder, AppHandle, WindowHandle, WindowOptions};
use creamui::{BoxedWidget, Size};

const WINDOW_WIDTH: u32 = 1120;
const WINDOW_HEIGHT: u32 = 720;
const MIN_WIDTH: u32 = 880;
const MIN_HEIGHT: u32 = 580;

#[cfg(windows)]
fn set_native_window_icon() {
    use windows::core::{w, PCWSTR};
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        FindWindowW, LoadImageW, SendMessageW, ICON_BIG, ICON_SMALL, IMAGE_ICON, LR_DEFAULTSIZE,
        LR_SHARED, WM_SETICON,
    };

    unsafe {
        let hwnd = match FindWindowW(None, w!("CreamPaper")) {
            Ok(hwnd) => hwnd,
            Err(error) => {
                log::error!("Could not find CreamPaper window to set its icon: {error}");
                return;
            }
        };
        let module = match GetModuleHandleW(None) {
            Ok(module) => module,
            Err(error) => {
                log::error!("Could not load the application module for its icon: {error}");
                return;
            }
        };
        let icon = match LoadImageW(
            module,
            PCWSTR(1usize as *const u16),
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE | LR_SHARED,
        ) {
            Ok(icon) => icon,
            Err(error) => {
                log::error!("Could not load the embedded application icon: {error}");
                return;
            }
        };

        SendMessageW(
            hwnd,
            WM_SETICON,
            WPARAM(ICON_SMALL as usize),
            LPARAM(icon.0 as isize),
        );
        SendMessageW(
            hwnd,
            WM_SETICON,
            WPARAM(ICON_BIG as usize),
            LPARAM(icon.0 as isize),
        );
    }
}

pub fn launch() {
    if engine::activate_existing_instance() {
        return;
    }
    let start_hidden = std::env::args().any(|arg| arg == "--background");
    let state = AppState::new(model::load());
    let theme = ui::theme::app_theme();

    let on_started = {
        let state = state.clone();
        move |app: AppHandle| state.attach(app)
    };

    let on_ready = {
        let state = state.clone();
        move |window: WindowHandle| {
            #[cfg(windows)]
            set_native_window_icon();

            window.set_min_size(Some((MIN_WIDTH, MIN_HEIGHT)));
            state.attach_window(window, start_hidden);
        }
    };

    let build_ui = move |viewport: Size| -> BoxedWidget { ui::root(state.clone(), viewport) };

    AppBuilder::new()
        .keep_running()
        .on_started(on_started)
        .window(
            WindowOptions {
                title: "CreamPaper".into(),
                width: WINDOW_WIDTH,
                height: WINDOW_HEIGHT,
                decorations: true,
                resizable: true,
                theme,
                ..Default::default()
            },
            theme.colors.surface,
            on_ready,
            build_ui,
        )
        .run();
}
