use crate::ui::theme;
use creamui::widgets::{RawScrollView, ScrollController};
use creamui::{component, BoxedWidget, Color, Style};

/// Transparent scroll container with a slim scrollbar.
#[component]
pub fn Scroll(
    controller: ScrollController,
    style: Style,
    children: Vec<BoxedWidget>,
) -> BoxedWidget {
    let muted = theme::TEXT_MUTED;
    Box::new(
        RawScrollView::controlled(style, controller)
            .scrollbar_color(Color::rgba(muted.r, muted.g, muted.b, 70))
            .scrollbar_hover_color(Color::rgba(muted.r, muted.g, muted.b, 160))
            .scrollbar_pressed_color(theme::TEXT)
            .with_children(children),
    )
}
