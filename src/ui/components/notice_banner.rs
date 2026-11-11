use super::*;
use crate::app::{Notice, NoticeKind};
use crate::ui::icons::{icon, AppIcon};
use crate::ui::theme::{self, row};
use creamui::{component, jsx, BoxedWidget, Color};

#[component]
pub fn NoticeBanner(notice: Notice, on_dismiss: Callback) -> BoxedWidget {
    let (tint, glyph) = match notice.kind {
        NoticeKind::Info => (theme::ACCENT_HOVER, AppIcon::Check),
        NoticeKind::Error => (theme::DANGER, AppIcon::Alert),
    };
    let style = row(12.0)
        .width("100%")
        .padding("10px 10px 10px 14px")
        .background(Color::rgba(tint.r, tint.g, tint.b, 26))
        .border(Color::rgba(tint.r, tint.g, tint.b, 70), 1.0)
        .corner_radius(12.0);
    Box::new(jsx! {
        <RawView style={style}>
            {icon(glyph, tint, 18.0)}
            <RawView style={creamui::Style::new().flex_grow(1.0)}>
                <RawText color={theme::TEXT} font_size={13.0}>{notice.text}</RawText>
            </RawView>
            <IconButton icon_name={AppIcon::Close} size={28.0} danger={false} on_click={on_dismiss} />
        </RawView>
    })
}
