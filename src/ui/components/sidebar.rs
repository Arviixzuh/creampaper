use super::*;
use crate::app::{AppState, Page};
use crate::ui::icons::{icon, AppIcon};
use crate::ui::theme::{self, centered_column, column, row, SIDEBAR_WIDTH};
use creamui::{component, jsx, BoxedWidget, Color};

const NAV: [(Page, AppIcon, &str); 3] = [
    (Page::Library, AppIcon::Library, "Biblioteca"),
    (Page::Settings, AppIcon::Settings, "Ajustes"),
    (Page::About, AppIcon::Info, "Acerca de"),
];

#[component]
pub fn Sidebar(state: AppState, viewport_height: f32) -> BoxedWidget {
    let current = state.page.get();
    let status = state.status.get();
    let count = state.library.get().len();

    let style = column(6.0)
        .width(SIDEBAR_WIDTH)
        .height(viewport_height)
        .flex_shrink(0.0)
        .padding("22px 14px 18px 14px")
        .background(theme::SIDEBAR)
        .border(theme::BORDER, 1.0);
    let logo = centered_column(0.0)
        .width(36.0)
        .height(36.0)
        .background(theme::BRAND_GRADIENT)
        .corner_radius(11.0);

    let items: Vec<BoxedWidget> = NAV
        .into_iter()
        .map(|(page, glyph, label)| {
            let nav = state.clone();
            jsx! {
                <NavItem icon_name={glyph} label={label.to_owned()} active={current == page}
                    on_click={callback(move || nav.navigate(page))} />
            }
        })
        .collect();
    let nav = creamui::widgets::RawView::new(column(4.0).width("100%")).with_children(items);

    Box::new(jsx! {
        <RawView style={style}>
            <RawView style={row(12.0).padding("0px 6px 22px 6px")}>
                <RawView style={logo}>{icon(AppIcon::MonitorPen, Color::rgb(29, 29, 29), 22.0)}</RawView>
                <RawView style={column(1.0)}>
                    <RawText color={theme::TEXT} font_size={16.0} bold={true}>"CreamPaper"</RawText>
                    <RawText color={theme::TEXT_FAINT} font_size={11.5}>"Fondos animados"</RawText>
                </RawView>
            </RawView>
            {Box::new(nav) as BoxedWidget}
            <RawView style={creamui::Style::new().flex_grow(1.0)} />
            <RawView style={column(10.0).width("100%").padding(12.0).background(theme::CARD).border(theme::BORDER, 1.0).corner_radius(12.0)}>
                <RawText color={theme::TEXT_FAINT} font_size={11.0} bold={true}>"MOTOR"</RawText>
                <StatusBadge status={status} />
                <RawText color={theme::TEXT_MUTED} font_size={12.0}>{format!("{count} videos en la biblioteca")}</RawText>
            </RawView>
        </RawView>
    })
}
