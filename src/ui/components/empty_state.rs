use super::*;
use crate::ui::icons::{icon, AppIcon};
use crate::ui::theme::{self, centered_column, optional};
use creamui::core::TextAlign;
use creamui::{component, jsx, BoxedWidget};

#[component]
pub fn EmptyState(title: String, message: String, on_add: Option<Callback>) -> BoxedWidget {
    let style = centered_column(14.0)
        .width("100%")
        .padding("56px 24px")
        .background(theme::CARD)
        .border(theme::BORDER, 1.0)
        .corner_radius(16.0);
    let badge = centered_column(0.0)
        .width(64.0)
        .height(64.0)
        .background(theme::ACCENT_SOFT)
        .corner_radius(20.0);
    let action = on_add.map(|on_add| -> BoxedWidget {
        jsx! { <PrimaryButton icon_name={Some(AppIcon::Plus)} label={"Añadir videos".to_owned()} on_click={on_add} /> }
    });
    Box::new(jsx! {
        <RawView style={style}>
            <RawView style={badge}>{icon(AppIcon::Film, theme::ACCENT_HOVER, 28.0)}</RawView>
            <RawText color={theme::TEXT} font_size={17.0} bold={true} align={TextAlign::Center}>{title}</RawText>
            <RawText color={theme::TEXT_MUTED} font_size={13.0} align={TextAlign::Center} max_width={420.0}>{message}</RawText>
            {optional(action)}
        </RawView>
    })
}
