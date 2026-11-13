//! Line icons drawn from inline SVG, rasterized once and cached per thread.
//! They are monochrome, so the same bitmap is tinted for any state.

use creamui::core::RgbaImage;
use creamui::widgets::{Icon, IconImage, IconSource};
use creamui::{BoxedWidget, Color};
use creamui_image::{ImageData, SvgSize};
use std::cell::RefCell;
use std::collections::HashMap;

const RASTER_SIZE: u32 = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AppIcon {
    Library,
    Settings,
    Info,
    Plus,
    Play,
    Pause,
    Stop,
    Trash,
    Search,
    Monitor,
    MonitorPen,
    Film,
    Zap,
    Battery,
    Volume,
    Power,
    Close,
    Check,
    Alert,
    Lang,
}

fn path_data(icon: AppIcon) -> &'static str {
    match icon {
        AppIcon::Library => {
            r#"<rect x="3" y="3" width="7" height="7" rx="1.5"/><rect x="14" y="3" width="7" height="7" rx="1.5"/><rect x="3" y="14" width="7" height="7" rx="1.5"/><rect x="14" y="14" width="7" height="7" rx="1.5"/>"#
        }
        AppIcon::Settings => {
            r#"<line x1="4" y1="6" x2="20" y2="6"/><line x1="4" y1="12" x2="20" y2="12"/><line x1="4" y1="18" x2="20" y2="18"/><circle cx="9" cy="6" r="2" fill="black"/><circle cx="15" cy="12" r="2" fill="black"/><circle cx="7" cy="18" r="2" fill="black"/>"#
        }
        AppIcon::Info => {
            r#"<circle cx="12" cy="12" r="9"/><line x1="12" y1="11" x2="12" y2="16"/><line x1="12" y1="8" x2="12.01" y2="8"/>"#
        }
        AppIcon::Plus => {
            r#"<line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>"#
        }
        AppIcon::Play => r#"<path d="M7 4.5v15l12-7.5z" fill="white"/>"#,
        AppIcon::Pause => {
            r#"<rect x="6" y="4.5" width="4" height="15" rx="1" fill="white"/><rect x="14" y="4.5" width="4" height="15" rx="1" fill="white"/>"#
        }
        AppIcon::Stop => r#"<rect x="5.5" y="5.5" width="13" height="13" rx="2" fill="white"/>"#,
        AppIcon::Trash => {
            r#"<path d="M4 7h16"/><path d="M9 7V4.5h6V7"/><path d="M6.5 7l1 13h9l1-13"/>"#
        }
        AppIcon::Search => {
            r#"<circle cx="11" cy="11" r="7"/><line x1="16.5" y1="16.5" x2="21" y2="21"/>"#
        }
        AppIcon::Monitor => {
            r#"<rect x="3" y="4" width="18" height="12" rx="2"/><line x1="8" y1="20" x2="16" y2="20"/><line x1="12" y1="16" x2="12" y2="20"/>"#
        }
        AppIcon::MonitorPen => {
            r#"<rect x="2" y="3" width="20" height="14" rx="2.5"/><line x1="12" y1="17" x2="12" y2="21"/><line x1="8" y1="21" x2="16" y2="21"/><path d="M8.5 13.5L10.6 13.1L15.35 8.35L13.65 6.65L8.9 11.4Z"/><line x1="10.6" y1="13.1" x2="8.9" y2="11.4"/><line x1="12.25" y1="8.05" x2="13.95" y2="9.75"/>"#
        }
        AppIcon::Film => {
            r#"<rect x="3" y="4" width="18" height="16" rx="2"/><line x1="7" y1="4" x2="7" y2="20"/><line x1="17" y1="4" x2="17" y2="20"/><line x1="3" y1="12" x2="21" y2="12"/>"#
        }
        AppIcon::Zap => r#"<path d="M13 2L4 14h7l-1 8 9-12h-7z"/>"#,
        AppIcon::Battery => {
            r#"<rect x="2" y="7" width="17" height="10" rx="2"/><line x1="22" y1="11" x2="22" y2="13"/><line x1="6" y1="10" x2="6" y2="14"/>"#
        }
        AppIcon::Volume => {
            r#"<path d="M11 5L6 9H3v6h3l5 4z"/><path d="M15.5 8.5a5 5 0 0 1 0 7"/><path d="M18.5 5.5a9 9 0 0 1 0 13"/>"#
        }
        AppIcon::Power => r#"<path d="M12 3v9"/><path d="M6.4 6.4a8 8 0 1 0 11.2 0"/>"#,
        AppIcon::Close => {
            r#"<line x1="6" y1="6" x2="18" y2="18"/><line x1="18" y1="6" x2="6" y2="18"/>"#
        }
        AppIcon::Check => r#"<path d="M5 12.5l4.5 4.5L19 7.5"/>"#,
        AppIcon::Alert => {
            r#"<circle cx="12" cy="12" r="9"/><line x1="12" y1="7.5" x2="12" y2="13"/><line x1="12" y1="16.5" x2="12.01" y2="16.5"/>"#
        }
        AppIcon::Lang => {
            r#"<circle cx="12" cy="12" r="9"/><path d="M3.6 9h16.8M3.6 15h16.8"/><path d="M12 3a15.3 15.3 0 0 1 4 9 15.3 15.3 0 0 1-4 9 15.3 15.3 0 0 1-4-9 15.3 15.3 0 0 1 4-9z"/>"#
        }
    }
}

fn rasterize(icon: AppIcon) -> Option<RgbaImage> {
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{}</svg>"#,
        path_data(icon)
    );
    ImageData::from_svg(svg.as_bytes(), SvgSize::Max(RASTER_SIZE))
        .ok()
        .map(|data| data.image().clone())
}

thread_local! {
    static CACHE: RefCell<HashMap<AppIcon, Option<RgbaImage>>> = RefCell::new(HashMap::new());
}

pub fn icon(icon: AppIcon, color: Color, size: f32) -> BoxedWidget {
    let image = CACHE.with(|cache| {
        cache
            .borrow_mut()
            .entry(icon)
            .or_insert_with(|| rasterize(icon))
            .clone()
    });
    match image {
        Some(image) => Box::new(
            Icon::new(
                IconSource::Image(IconImage {
                    image,
                    monochrome: true,
                }),
                color,
            )
            .size(size),
        ),
        None => Box::new(Icon::new(creamui::widgets::Symbol::Image, color).size(size)),
    }
}
