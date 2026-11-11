use crate::ui::theme::{self, column};
use creamui::widgets::RawView;
use creamui::{component, jsx, BoxedWidget};

#[component]
pub fn SettingsSection(title: String, children: Vec<BoxedWidget>) -> BoxedWidget {
    let card = column(0.0)
        .width("100%")
        .background(theme::CARD)
        .border(theme::BORDER, 1.0)
        .corner_radius(14.0);
    let body: BoxedWidget = Box::new(RawView::new(card).with_children(children));
    Box::new(jsx! {
        <RawView style={column(10.0).width("100%")}>
            <RawText color={theme::TEXT_FAINT} font_size={11.5} bold={true}>{title.to_uppercase()}</RawText>
            {body}
        </RawView>
    })
}
