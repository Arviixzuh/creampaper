//! Plain data types and their persistence. Nothing here depends on the UI.

mod settings;
mod storage;
mod wallpaper;

pub use settings::{FitMode, MonitorMode, Settings};
pub use storage::{load, save, AppData};
pub use wallpaper::{Wallpaper, VIDEO_EXTENSIONS};
