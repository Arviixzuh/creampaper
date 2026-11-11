use serde::{Deserialize, Serialize};

/// How a video is scaled to fill each monitor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FitMode {
    /// Fills the monitor, cropping the edges that do not fit.
    #[default]
    Cover,
    /// Shows the whole video, adding black bars when needed.
    Contain,
    /// Fills the monitor ignoring the aspect ratio.
    Stretch,
}

impl FitMode {
    pub const ALL: [FitMode; 3] = [FitMode::Cover, FitMode::Contain, FitMode::Stretch];

    pub fn label(self) -> &'static str {
        match self {
            FitMode::Cover => "Cubrir",
            FitMode::Contain => "Ajustar",
            FitMode::Stretch => "Estirar",
        }
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|mode| *mode == self).unwrap_or(0)
    }

    pub fn from_index(index: usize) -> Self {
        Self::ALL.get(index).copied().unwrap_or_default()
    }
}

/// Which monitors display the wallpaper.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonitorMode {
    #[default]
    All,
    Primary,
}

impl MonitorMode {
    pub const ALL: [MonitorMode; 2] = [MonitorMode::All, MonitorMode::Primary];

    pub fn label(self) -> &'static str {
        match self {
            MonitorMode::All => "Todos",
            MonitorMode::Primary => "Principal",
        }
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|mode| *mode == self).unwrap_or(0)
    }

    pub fn from_index(index: usize) -> Self {
        Self::ALL.get(index).copied().unwrap_or_default()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub fit: FitMode,
    pub monitors: MonitorMode,
    pub muted: bool,
    /// Linear volume in `0.0..=1.0`.
    pub volume: f32,
    pub pause_on_fullscreen: bool,
    pub pause_on_maximized: bool,
    pub pause_on_battery: bool,
    pub start_with_windows: bool,
    pub restore_on_launch: bool,
    pub close_to_tray: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            fit: FitMode::Cover,
            monitors: MonitorMode::All,
            muted: true,
            volume: 0.5,
            pause_on_fullscreen: true,
            pause_on_maximized: false,
            pause_on_battery: true,
            start_with_windows: false,
            restore_on_launch: true,
            close_to_tray: true,
        }
    }
}
