use crate::engine::{self, EngineCommand, EngineEvent, EngineHandle, PlaybackStatus};
use crate::i18n;
use crate::model::{self, AppData, Settings, Wallpaper, VIDEO_EXTENSIONS};
use crate::tr;
use creamui::render::{AppHandle, WindowHandle};
use creamui::widgets::{ScrollController, SelectController};
use creamui::{ImageData, Signal};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

const THUMB_WIDTH: u32 = 480;
const THUMB_HEIGHT: u32 = 270;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    Library,
    Settings,
    About,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoticeKind {
    Info,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Notice {
    pub kind: NoticeKind,
    pub text: String,
}

/// UI state shared by every component. Cloning is cheap: every field is a
/// reactive handle or a reference-counted pointer.
#[derive(Clone)]
pub struct AppState {
    pub page: Signal<Page>,
    pub settings: Signal<Settings>,
    pub library: Signal<Vec<Wallpaper>>,
    pub active: Signal<Option<u64>>,
    pub status: Signal<PlaybackStatus>,
    pub thumbnails: Signal<HashMap<u64, ImageData>>,
    pub search: Signal<String>,
    pub notice: Signal<Option<Notice>>,
    pub scroll: ScrollController,
    pub lang_select: SelectController,
    /// Toggled to repaint every pixel (see `force_full_repaint`).
    pub repaint: Signal<bool>,
    app: Rc<RefCell<Option<AppHandle>>>,
    window: Rc<RefCell<Option<WindowHandle>>>,
    engine: Rc<RefCell<Option<EngineHandle>>>,
}

impl AppState {
    pub fn new(data: AppData) -> Self {
        i18n::set(data.settings.lang);
        let lang_select = SelectController::new(data.settings.lang.index());
        let active = data
            .active
            .filter(|id| data.library.iter().any(|item| item.id == *id));
        AppState {
            page: Signal::new(Page::Library),
            settings: Signal::new(data.settings),
            library: Signal::new(data.library),
            active: Signal::new(active),
            status: Signal::new(PlaybackStatus::Idle),
            thumbnails: Signal::new(HashMap::new()),
            search: Signal::new(String::new()),
            notice: Signal::new(None),
            scroll: ScrollController::default(),
            lang_select,
            repaint: Signal::new(false),
            app: Rc::new(RefCell::new(None)),
            window: Rc::new(RefCell::new(None)),
            engine: Rc::new(RefCell::new(None)),
        }
    }

    /// Starts the engine once the event loop is running.
    pub fn attach(&self, app: AppHandle) {
        let events = self.clone();
        let handle = engine::start(&app, self.settings.peek(), move |event| {
            events.handle_engine_event(event)
        });
        *self.engine.borrow_mut() = Some(handle);
        *self.app.borrow_mut() = Some(app);

        for wallpaper in self.library.peek() {
            self.load_thumbnail(&wallpaper);
        }
        if self.settings.peek().restore_on_launch {
            if let Some(wallpaper) = self.active_wallpaper().filter(Wallpaper::exists) {
                self.send(EngineCommand::Play(wallpaper.path));
            }
        } else {
            self.active.set(None);
        }
    }

    pub fn attach_window(&self, window: WindowHandle, start_hidden: bool) {
        let dropped = self.clone();
        window.on_files_dropped(move |paths| dropped.add_files(paths));
        let closing = self.clone();
        window.on_close_requested(move || closing.on_close_requested());
        let restored = self.clone();
        engine::on_window_restored(&window, move || restored.force_full_repaint());
        if start_hidden {
            window.hide();
        }
        *self.window.borrow_mut() = Some(window);
    }

    fn send(&self, command: EngineCommand) {
        if let Some(engine) = self.engine.borrow().as_ref() {
            engine.send(command);
        }
    }

    fn handle_engine_event(&self, event: EngineEvent) {
        match event {
            EngineEvent::Status(status) => self.status.set_if_changed(status),
            EngineEvent::Error(text) => self.notify(NoticeKind::Error, text),
            EngineEvent::ShowWindow => self.show_window(),
            EngineEvent::Stopped => {
                self.active.set(None);
                self.persist();
            }
            EngineEvent::QuitRequested => self.quit(),
            EngineEvent::Exited => {
                if let Some(app) = self.app.borrow().as_ref() {
                    app.exit();
                }
            }
        }
    }

    pub fn active_wallpaper(&self) -> Option<Wallpaper> {
        let id = self.active.get()?;
        self.library.get().into_iter().find(|item| item.id == id)
    }

    pub fn navigate(&self, page: Page) {
        self.page.set_if_changed(page);
        self.scroll.set(0.0);
    }

    pub fn notify(&self, kind: NoticeKind, text: impl Into<String>) {
        self.notice.set(Some(Notice {
            kind,
            text: text.into(),
        }));
    }

    pub fn dismiss_notice(&self) {
        self.notice.set(None);
    }

    pub fn pick_files(&self) {
        let picked = rfd::FileDialog::new()
            .set_title(tr!("file_dialog.title"))
            .add_filter(&tr!("file_dialog.videos"), VIDEO_EXTENSIONS)
            .pick_files();
        if let Some(paths) = picked {
            self.add_files(paths);
        }
    }

    pub fn add_files(&self, paths: Vec<PathBuf>) {
        let mut library = self.library.peek();
        let mut next_id = library.iter().map(|item| item.id).max().unwrap_or(0) + 1;
        let mut added = Vec::new();
        let mut rejected = 0;
        for path in paths {
            if library.iter().any(|item| item.path == path) {
                continue;
            }
            match Wallpaper::from_path(next_id, &path) {
                Some(wallpaper) => {
                    next_id += 1;
                    library.push(wallpaper.clone());
                    added.push(wallpaper);
                }
                None => rejected += 1,
            }
        }
        if !added.is_empty() {
            self.library.set(library);
            self.persist();
            for wallpaper in &added {
                self.load_thumbnail(wallpaper);
            }
            let message = if added.len() == 1 {
                tr!("notice.files_added.one", count = added.len())
            } else {
                tr!("notice.files_added.many", count = added.len())
            };
            self.notify(NoticeKind::Info, message);
        }
        if rejected > 0 {
            self.notify(
                NoticeKind::Error,
                tr!(
                    "notice.files_rejected",
                    count = rejected,
                    formats = VIDEO_EXTENSIONS.join(", ")
                ),
            );
        }
    }

    pub fn apply(&self, id: u64) {
        let Some(wallpaper) = self.library.peek().into_iter().find(|item| item.id == id) else {
            return;
        };
        if !wallpaper.exists() {
            self.notify(
                NoticeKind::Error,
                tr!("notice.file_missing", path = wallpaper.path.display()),
            );
            return;
        }
        self.active.set(Some(id));
        self.send(EngineCommand::Play(wallpaper.path));
        self.persist();
        if !engine::SUPPORTED {
            self.notify(NoticeKind::Info, tr!("notice.preview_only"));
        }
    }

    pub fn remove(&self, id: u64) {
        if self.active.peek() == Some(id) {
            self.stop();
        }
        self.library
            .update(|library| library.retain(|item| item.id != id));
        self.thumbnails.update(|thumbnails| {
            thumbnails.remove(&id);
        });
        self.persist();
    }

    pub fn toggle_pause(&self) {
        match self.status.peek() {
            PlaybackStatus::Playing => self.send(EngineCommand::Pause),
            PlaybackStatus::Paused(_) => self.send(EngineCommand::Resume),
            PlaybackStatus::Idle => {}
        }
    }

    pub fn stop(&self) {
        self.active.set(None);
        self.send(EngineCommand::Stop);
        self.persist();
    }

    pub fn update_settings(&self, change: impl FnOnce(&mut Settings)) {
        let previous = self.settings.peek();
        let mut next = previous.clone();
        change(&mut next);
        if next == previous {
            return;
        }
        if next.start_with_windows != previous.start_with_windows {
            if let Err(error) = engine::set_autostart(next.start_with_windows) {
                next.start_with_windows = previous.start_with_windows;
                self.notify(NoticeKind::Error, error);
            }
        }
        if next.lang != previous.lang {
            i18n::set(next.lang);
        }
        self.settings.set(next.clone());
        self.send(EngineCommand::Configure(next));
        self.persist();
    }

    pub fn show_window(&self) {
        if let Some(window) = self.window.borrow().as_ref() {
            window.show();
        }
        #[cfg(not(windows))]
        self.force_full_repaint();
    }

    /// CreamUI only presents the regions that changed since the last frame,
    /// but the OS drops the window contents while it is minimized or hidden.
    /// Flipping the root background between two indistinguishable colors
    /// makes the next frame cover the whole window.
    pub fn force_full_repaint(&self) {
        self.repaint.update(|flag| *flag = !*flag);
    }

    pub fn quit(&self) {
        if self.engine.borrow().is_some() {
            self.send(EngineCommand::Shutdown);
        } else if let Some(app) = self.app.borrow().as_ref() {
            app.exit();
        }
    }

    fn on_close_requested(&self) -> bool {
        if engine::SUPPORTED && self.settings.peek().close_to_tray {
            if let Some(window) = self.window.borrow().as_ref() {
                window.hide();
            }
        } else {
            self.quit();
        }
        false
    }

    fn load_thumbnail(&self, wallpaper: &Wallpaper) {
        let Some(app) = self.app.borrow().clone() else {
            return;
        };
        let id = wallpaper.id;
        let path = wallpaper.path.clone();
        let thumbnails = self.thumbnails.clone();
        app.spawn_background(
            move || engine::thumbnail(&path, THUMB_WIDTH, THUMB_HEIGHT),
            move |thumbnail| {
                let image = thumbnail.and_then(|thumbnail| {
                    ImageData::from_rgba(thumbnail.width, thumbnail.height, thumbnail.rgba).ok()
                });
                if let Some(image) = image {
                    thumbnails.update(|map| {
                        map.insert(id, image);
                    });
                }
            },
        );
    }

    fn persist(&self) {
        model::save(&AppData {
            settings: self.settings.peek(),
            library: self.library.peek(),
            active: self.active.peek(),
        });
    }
}
