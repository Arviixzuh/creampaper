use super::*;
use crate::model::Wallpaper;
use crate::ui::icons::{icon, AppIcon};
use crate::ui::theme::{self, column, ellipsize, optional, row};
use creamui::core::layout::{JustifyContent, Position};
use creamui::{component, jsx, BoxedWidget, Color, ImageData, StateStyle};

/// Fixed footer height: cards never depend on intrinsic text measurement,
/// so a freshly added card lays out exactly like the others.
const FOOTER_HEIGHT: f32 = 58.0;

#[component]
pub fn WallpaperCard(
    wallpaper: Wallpaper,
    thumbnail: Option<ImageData>,
    width: f32,
    active: bool,
    on_apply: Callback,
    on_remove: Callback,
) -> BoxedWidget {
    let image_height = (width - 2.0) * 9.0 / 16.0;
    let border = if active { theme::ACCENT } else { theme::BORDER };
    let card = column(0.0)
        .width(width)
        .height(image_height + FOOTER_HEIGHT + 2.0)
        .background(theme::CARD)
        .border(border, if active { 2.0 } else { 1.0 })
        .corner_radius(14.0)
        .hover(StateStyle::new().background(theme::CARD_HOVER).border(
            if active {
                theme::ACCENT
            } else {
                theme::BORDER_STRONG
            },
            if active { 2.0 } else { 1.0 },
        ));
    let active_badge = active.then(|| -> BoxedWidget {
        Box::new(jsx! {
            <RawView style={row(6.0).position(Position::Absolute).top(10.0).left(10.0).height(24.0).padding("0px 9px").background(theme::ACCENT).corner_radius(12.0)}>
                {icon(AppIcon::Check, Color::rgb(255, 255, 255), 12.0)}
                <RawText color={Color::rgb(255, 255, 255)} font_size={11.5} bold={true}>"Activo"</RawText>
            </RawView>
        })
    });
    let missing = !wallpaper.exists();
    let meta = if missing {
        "Archivo no encontrado".to_owned()
    } else {
        format!("{} · {}", wallpaper.format_label(), wallpaper.size_label())
    };
    let meta_color = if missing {
        theme::DANGER
    } else {
        theme::TEXT_FAINT
    };
    let name = ellipsize(&wallpaper.name, ((width - 70.0) / 7.5).max(8.0) as usize);

    Box::new(jsx! {
        <RawView style={card}>
            <RawButton style={column(0.0).width(width - 2.0).height(image_height)} on_click={move || on_apply()}>
                <Thumbnail image={thumbnail} width={width - 2.0} height={image_height} radius={13.0} />
                {optional(active_badge)}
            </RawButton>
            <RawView style={row(10.0).width("100%").height(FOOTER_HEIGHT).padding("0px 10px 0px 14px").justify_content(JustifyContent::SpaceBetween)}>
                <RawView style={column(3.0).flex_grow(1.0).flex_shrink(1.0)}>
                    <RawText color={theme::TEXT} font_size={13.5} bold={true}>{name}</RawText>
                    <RawText color={meta_color} font_size={11.5}>{meta}</RawText>
                </RawView>
                <IconButton icon_name={AppIcon::Trash} size={32.0} danger={true} on_click={on_remove} />
            </RawView>
        </RawView>
    })
}
