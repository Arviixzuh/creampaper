use super::{Settings, Wallpaper};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const APP_DIR: &str = "CreamPaper";
const FILE_NAME: &str = "config.json";

/// Everything persisted between launches, stored as one small JSON file.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppData {
    pub settings: Settings,
    pub library: Vec<Wallpaper>,
    pub active: Option<u64>,
}

fn config_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(std::env::temp_dir);
    base.join(APP_DIR)
}

pub fn load() -> AppData {
    std::fs::read(config_dir().join(FILE_NAME))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

/// Writes through a temporary file so a crash never leaves a truncated config.
pub fn save(data: &AppData) {
    let dir = config_dir();
    let result = std::fs::create_dir_all(&dir).and_then(|()| {
        let json = serde_json::to_vec_pretty(data).map_err(std::io::Error::other)?;
        let temporary = dir.join(format!("{FILE_NAME}.tmp"));
        std::fs::write(&temporary, json)?;
        std::fs::rename(temporary, dir.join(FILE_NAME))
    });
    if let Err(error) = result {
        eprintln!("[creampaper] could not save settings: {error}");
    }
}
