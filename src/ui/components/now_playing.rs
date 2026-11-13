use super::*;
use crate::app::AppState;
use crate::engine::PlaybackStatus;
use crate::tr;
use crate::ui::icons::AppIcon;
use crate::ui::theme::{self, column, ellipsize, row};
use creamui::core::layout::JustifyContent;
use creamui::{component, jsx, BoxedWidget};

/// Hero card at the top of the library with the live wallpaper and controls.
#[component]
pub fn NowPlaying(state: AppState) -> BoxedWidget {
    let _language = state.settings.get().lang;
    let status = state.status.get();
    let active = state.active_wallpaper();
    let thumbnail = active
        .as_ref()
        .and_then(|item| state.thumbnails.get().get(&item.id).cloned());

    let style = row(18.0)
        .width("100%")
        .padding(14.0)
        .background(theme::HERO_GRADIENT)
        .border(theme::BORDER, 1.0)
        .corner_radius(16.0);

    let (title, detail) = match &active {
        Some(item) => (
            ellipsize(&item.name, 48),
            format!("{} · {}", item.format_label(), item.size_label()),
        ),
        None => (
            tr!("now_playing.empty.title"),
            tr!("now_playing.empty.message"),
        ),
    };

    let controls: BoxedWidget = if active.is_some() {
        let pause_icon = if status == PlaybackStatus::Playing {
            AppIcon::Pause
        } else {
            AppIcon::Play
        };
        let toggle = state.clone();
        let stop = state.clone();
        Box::new(jsx! {
            <RawView style={row(8.0).flex_shrink(0.0)}>
                <IconButton icon_name={pause_icon} size={40.0} danger={false} on_click={callback(move || toggle.toggle_pause())} />
                <IconButton icon_name={AppIcon::Stop} size={40.0} danger={false} on_click={callback(move || stop.stop())} />
            </RawView>
        })
    } else {
        Box::new(creamui::widgets::RawView::new(creamui::Style::new()))
    };

    Box::new(jsx! {
        <RawView style={style}>
            <Thumbnail image={thumbnail} width={168.0} height={94.5} radius={10.0} />
            <RawView style={column(6.0).flex_grow(1.0).flex_shrink(1.0).justify_content(JustifyContent::Center)}>
                <RawText color={theme::TEXT_FAINT} font_size={11.0} bold={true}>{tr!("now_playing.label")}</RawText>
                <RawText color={theme::TEXT} font_size={18.0} bold={true}>{title}</RawText>
                <RawText color={theme::TEXT_MUTED} font_size={12.5}>{detail}</RawText>
                <RawView style={row(0.0).padding_top(4.0)}><StatusBadge status={status} /></RawView>
            </RawView>
            {controls}
        </RawView>
    })
}
