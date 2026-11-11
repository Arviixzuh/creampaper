use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Containers Media Foundation can decode on a stock Windows 10/11 install.
pub const VIDEO_EXTENSIONS: &[&str] = &["mp4", "m4v", "mov", "wmv", "avi", "mkv", "webm"];

pub fn is_video(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| VIDEO_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Wallpaper {
    pub id: u64,
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub added_at: u64,
}

impl Wallpaper {
    pub fn from_path(id: u64, path: &Path) -> Option<Self> {
        if !is_video(path) {
            return None;
        }
        let metadata = std::fs::metadata(path).ok().filter(|meta| meta.is_file())?;
        let name = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().replace(['_', '-'], " "))
            .unwrap_or_else(|| "Sin nombre".into());
        let added_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs());
        Some(Wallpaper {
            id,
            name,
            path: path.to_path_buf(),
            size_bytes: metadata.len(),
            added_at,
        })
    }

    pub fn exists(&self) -> bool {
        self.path.is_file()
    }

    pub fn format_label(&self) -> String {
        self.path
            .extension()
            .map(|ext| ext.to_string_lossy().to_ascii_uppercase())
            .unwrap_or_default()
    }

    pub fn size_label(&self) -> String {
        const MB: f64 = 1024.0 * 1024.0;
        let megabytes = self.size_bytes as f64 / MB;
        if megabytes >= 1024.0 {
            format!("{:.1} GB", megabytes / 1024.0)
        } else {
            format!("{megabytes:.1} MB")
        }
    }

    pub fn matches(&self, query: &str) -> bool {
        query.is_empty() || self.name.to_lowercase().contains(&query.to_lowercase())
    }
}
