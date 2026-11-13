use crate::engine::{PauseReason, PlaybackStatus};
use crate::tr;
use crate::ui::theme::{self, row};
use creamui::{component, jsx, BoxedWidget, Color};

pub fn status_label(status: PlaybackStatus) -> (String, Color) {
    match status {
        PlaybackStatus::Idle => (tr!("status.idle"), theme::TEXT_FAINT),
        PlaybackStatus::Playing => (tr!("status.playing"), theme::SUCCESS),
        PlaybackStatus::Paused(PauseReason::User) => (tr!("status.paused"), theme::WARNING),
        PlaybackStatus::Paused(PauseReason::Fullscreen) => {
            (tr!("status.paused.fullscreen"), theme::WARNING)
        }
        PlaybackStatus::Paused(PauseReason::Battery) => {
            (tr!("status.paused.battery"), theme::WARNING)
        }
    }
}

#[component]
pub fn StatusBadge(status: PlaybackStatus) -> BoxedWidget {
    let (label, color) = status_label(status);
    let soft = Color::rgba(color.r, color.g, color.b, 30);
    Box::new(jsx! {
        <RawView style={row(8.0).height(26.0).padding("0px 10px").background(soft).corner_radius(13.0)}>
            <RawView style={creamui::Style::new().width(7.0).height(7.0).background(color).corner_radius(4.0)} />
            <RawText color={color} font_size={12.0} bold={true}>{label}</RawText>
        </RawView>
    })
}
