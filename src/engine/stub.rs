//! Non-Windows fallback: keeps the UI fully usable for development while
//! simulating playback state.

use super::{EngineCommand, EngineEvent, PauseReason, PlaybackStatus, Thumbnail};
use crate::model::Settings;
use std::path::Path;
use std::sync::atomic::AtomicU32;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

pub fn wake(_thread_id: &AtomicU32) {}

pub fn run(
    commands: Receiver<EngineCommand>,
    _settings: Settings,
    emit: Box<dyn Fn(EngineEvent) + Send>,
    _thread_id: Arc<AtomicU32>,
) {
    let mut playing = false;
    while let Ok(command) = commands.recv() {
        match command {
            EngineCommand::Configure(_) => {}
            EngineCommand::Play(_) | EngineCommand::Resume => {
                playing = true;
                emit(EngineEvent::Status(PlaybackStatus::Playing));
            }
            EngineCommand::Pause if playing => {
                emit(EngineEvent::Status(PlaybackStatus::Paused(
                    PauseReason::User,
                )));
            }
            EngineCommand::Pause => {}
            EngineCommand::Stop => {
                playing = false;
                emit(EngineEvent::Status(PlaybackStatus::Idle));
            }
            EngineCommand::Shutdown => break,
        }
    }
    emit(EngineEvent::Exited);
}

/// Uses `ffmpeg` when it is on `PATH` so previews look real during development.
pub fn thumbnail(path: &Path, width: u32, height: u32) -> Option<Thumbnail> {
    let filter = format!(
        "scale={width}:{height}:force_original_aspect_ratio=increase,crop={width}:{height}"
    );
    let output = std::process::Command::new("ffmpeg")
        .args(["-loglevel", "error", "-ss", "1", "-i"])
        .arg(path)
        .args([
            "-frames:v",
            "1",
            "-vf",
            &filter,
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-",
        ])
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    let expected = (width * height * 4) as usize;
    (output.status.success() && output.stdout.len() == expected).then_some(Thumbnail {
        width,
        height,
        rgba: output.stdout,
    })
}

pub fn set_autostart(_enabled: bool) -> Result<(), String> {
    Err("El inicio automático solo está disponible en Windows".into())
}

pub fn on_window_restored(window: &creamui::render::WindowHandle, on_restore: impl Fn() + 'static) {
    window.on_pointer_presence(move |inside| {
        if inside {
            on_restore();
        }
    });
}

pub fn activate_existing_instance() -> bool {
    false
}
