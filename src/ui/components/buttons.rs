use super::*;
use crate::ui::icons::{icon, AppIcon};
use crate::ui::theme::{self, optional, row};
use creamui::core::layout::JustifyContent;
use creamui::{component, jsx, BoxedWidget, Color, StateStyle};

/// Filled accent button for the main action of a view.
#[component]
pub fn PrimaryButton(icon_name: Option<AppIcon>, label: String, on_click: Callback) -> BoxedWidget {
    let style = row(8.0)
        .height(38.0)
        .padding("0px 16px")
        .justify_content(JustifyContent::Center)
        .background(theme::ACCENT)
        .corner_radius(10.0)
        .hover(StateStyle::new().background(theme::ACCENT_HOVER))
        .pressed(StateStyle::new().background(theme::ACCENT_PRESSED));
    let leading = icon_name.map(|name| icon(name, Color::rgb(255, 255, 255), 16.0));
    Box::new(jsx! {
        <RawButton style={style} on_click={move || on_click()}>
            {optional(leading)}
            <RawText color={Color::rgb(255, 255, 255)} font_size={13.5} bold={true}>{label}</RawText>
        </RawButton>
    })
}

/// Subtle bordered button for secondary actions.
#[component]
pub fn GhostButton(icon_name: Option<AppIcon>, label: String, on_click: Callback) -> BoxedWidget {
    let style = row(8.0)
        .height(38.0)
        .padding("0px 14px")
        .justify_content(JustifyContent::Center)
        .background(theme::FIELD)
        .border(theme::BORDER, 1.0)
        .corner_radius(10.0)
        .hover(
            StateStyle::new()
                .background(theme::CARD_HOVER)
                .border(theme::BORDER_STRONG, 1.0),
        );
    let leading = icon_name.map(|name| icon(name, theme::TEXT, 16.0));
    Box::new(jsx! {
        <RawButton style={style} on_click={move || on_click()}>
            {optional(leading)}
            <RawText color={theme::TEXT} font_size={13.5}>{label}</RawText>
        </RawButton>
    })
}

/// Square icon-only button; `danger` tints the hover state red.
#[component]
pub fn IconButton(icon_name: AppIcon, size: f32, danger: bool, on_click: Callback) -> BoxedWidget {
    let hover = if danger {
        theme::DANGER_SOFT
    } else {
        theme::CARD_HOVER
    };
    let tint = if danger { theme::DANGER } else { theme::TEXT };
    let style = row(0.0)
        .width(size)
        .height(size)
        .justify_content(JustifyContent::Center)
        .background(theme::FIELD)
        .border(theme::BORDER, 1.0)
        .corner_radius(size * 0.3)
        .hover(
            StateStyle::new()
                .background(hover)
                .border(theme::BORDER_STRONG, 1.0),
        );
    Box::new(jsx! {
        <RawButton style={style} on_click={move || on_click()}>
            {icon(icon_name, tint, size * 0.45)}
        </RawButton>
    })
}
