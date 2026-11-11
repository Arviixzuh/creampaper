use crate::ui::icons::{icon, AppIcon};
use crate::ui::theme::{self, centered_column};
use creamui::{component, jsx, BoxedWidget, ImageData, ImageFit};

/// A 16:9 preview, or a gradient placeholder while Explorer has none.
#[component]
pub fn Thumbnail(image: Option<ImageData>, width: f32, height: f32, radius: f32) -> BoxedWidget {
    match image {
        Some(data) => Box::new(jsx! {
            <Image data={data} fit={ImageFit::Cover} corner_radius={radius} width={width} height={height} />
        }),
        None => Box::new(jsx! {
            <RawView style={centered_column(0.0).width(width).height(height).background(theme::PLACEHOLDER_GRADIENT).corner_radius(radius)}>
                {icon(AppIcon::Film, theme::TEXT_FAINT, (height * 0.28).clamp(18.0, 40.0))}
            </RawView>
        }),
    }
}
