use crate::engine::{PauseReason, PlaybackStatus};
use crate::ui::theme::{self, row};
use creamui::{component, jsx, BoxedWidget, Color};

pub fn status_label(status: PlaybackStatus) -> (&'static str, Color) {
    match status {
        PlaybackStatus::Idle => ("Inactivo", theme::TEXT_FAINT),
        PlaybackStatus::Playing => ("Reproduciendo", theme::SUCCESS),
        PlaybackStatus::Paused(PauseReason::User) => ("En pausa", theme::WARNING),
        PlaybackStatus::Paused(PauseReason::Fullscreen) => {
            ("Pausa · pantalla completa", theme::WARNING)
        }
        PlaybackStatus::Paused(PauseReason::Battery) => ("Pausa · batería", theme::WARNING),
    }
}

#[component]
pub fn StatusBadge(status: PlaybackStatus) -> BoxedWidget {
    let (label, color) = status_label(status);
    let soft = Color::rgba(color.r, color.g, color.b, 30);
    Box::new(jsx! {
        <RawView style={row(8.0).height(26.0).padding("0px 10px").background(soft).corner_radius(13.0)}>
            <RawView style={creamui::Style::new().width(7.0).height(7.0).background(color).corner_radius(4.0)} />
            <RawText color={color} font_size={12.0} bold={true}>{label.to_owned()}</RawText>
        </RawView>
    })
}
