use crate::ui::theme::{self, column, row};
use creamui::core::layout::JustifyContent;
use creamui::{component, jsx, BoxedWidget};

#[component]
pub fn PageHeader(title: String, subtitle: String, actions: Vec<BoxedWidget>) -> BoxedWidget {
    let style = row(16.0)
        .width("100%")
        .justify_content(JustifyContent::SpaceBetween);
    let actions_view = creamui::widgets::RawView::new(row(10.0)).with_children(actions);
    Box::new(jsx! {
        <RawView style={style}>
            <RawView style={column(4.0).flex_grow(1.0)}>
                <RawText color={theme::TEXT} font_size={26.0} bold={true}>{title}</RawText>
                <RawText color={theme::TEXT_MUTED} font_size={13.5}>{subtitle}</RawText>
            </RawView>
            {Box::new(actions_view) as BoxedWidget}
        </RawView>
    })
}
