use crate::ui::icons::{icon, AppIcon};
use crate::ui::theme::{self, column, row};
use creamui::core::layout::JustifyContent;
use creamui::{component, jsx, BoxedWidget};

#[component]
pub fn SettingRow(
    icon_name: AppIcon,
    title: String,
    description: String,
    control: BoxedWidget,
) -> BoxedWidget {
    let style = row(14.0)
        .width("100%")
        .padding("14px 16px")
        .justify_content(JustifyContent::SpaceBetween);
    let badge = row(0.0)
        .width(36.0)
        .height(36.0)
        .flex_shrink(0.0)
        .justify_content(JustifyContent::Center)
        .background(theme::FIELD)
        .corner_radius(10.0);
    Box::new(jsx! {
        <RawView style={style}>
            <RawView style={badge}>{icon(icon_name, theme::ACCENT_HOVER, 18.0)}</RawView>
            <RawView style={column(3.0).flex_grow(1.0).flex_shrink(1.0)}>
                <RawText color={theme::TEXT} font_size={14.0}>{title}</RawText>
                <RawText color={theme::TEXT_MUTED} font_size={12.5}>{description}</RawText>
            </RawView>
            <RawView style={row(0.0).flex_shrink(0.0)}>{control}</RawView>
        </RawView>
    })
}
