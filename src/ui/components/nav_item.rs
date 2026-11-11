use super::*;
use crate::ui::icons::{icon, AppIcon};
use crate::ui::theme::{self, row};
use creamui::{component, jsx, BoxedWidget, StateStyle};

#[component]
pub fn NavItem(icon_name: AppIcon, label: String, active: bool, on_click: Callback) -> BoxedWidget {
    let (background, tint, text) = if active {
        (theme::ACCENT_SOFT, theme::ACCENT_HOVER, theme::TEXT)
    } else {
        (
            creamui::Color::rgba(0, 0, 0, 0),
            theme::TEXT_MUTED,
            theme::TEXT_MUTED,
        )
    };
    let style = row(12.0)
        .width("100%")
        .height(40.0)
        .padding("0px 12px")
        .background(background)
        .corner_radius(10.0)
        .hover(StateStyle::new().background(if active {
            theme::ACCENT_SOFT
        } else {
            theme::CARD
        }));
    Box::new(jsx! {
        <RawButton style={style} on_click={move || on_click()}>
            {icon(icon_name, tint, 18.0)}
            <RawText color={text} font_size={14.0} bold={active}>{label}</RawText>
        </RawButton>
    })
}
