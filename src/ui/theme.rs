//! Visual language: one palette, a CreamUI `Theme` derived from it so the
//! stock controls match, and small layout helpers used across components.

use creamui::core::layout::{AlignItems, Display, FlexDirection, JustifyContent};
use creamui::widgets::RawView;
use creamui::{BoxedWidget, Color, ColorScheme, Style, Theme};

pub const SIDEBAR_WIDTH: f32 = 232.0;
pub const TITLEBAR_HEIGHT: f32 = 36.0;
pub const PAGE_PADDING: f32 = 32.0;
pub const SCROLLBAR_ALLOWANCE: f32 = 14.0;

pub const BACKGROUND: Color = Color::rgba(0x00, 0x00, 0x00, 110);
pub const BACKGROUND_REPAINT: Color = Color::rgba(0x00, 0x00, 0x00, 111);
pub const SURFACE: Color = Color::rgba(0x08, 0x08, 0x08, 180);
pub const SIDEBAR: Color = Color::rgba(0x0a, 0x0a, 0x0a, 215);
pub const CARD: Color = Color::rgba(0x0a, 0x0a, 0x0a, 195);
pub const CARD_HOVER: Color = Color::rgba(0x14, 0x14, 0x14, 220);
pub const FIELD: Color = Color::rgba(0x11, 0x11, 0x11, 205);
pub const BORDER: Color = Color::rgba(255, 255, 255, 26);
pub const BORDER_STRONG: Color = Color::rgba(255, 255, 255, 51);

pub const ACCENT: Color = Color::rgb(0x00, 0x70, 0xf3);
pub const ACCENT_HOVER: Color = Color::rgb(0x2a, 0x8b, 0xff);
pub const ACCENT_PRESSED: Color = Color::rgb(0x00, 0x5b, 0xc4);
pub const ACCENT_SOFT: Color = Color::rgba(0x00, 0x70, 0xf3, 38);

pub const TEXT: Color = Color::rgb(0xed, 0xed, 0xed);
pub const TEXT_MUTED: Color = Color::rgb(0xa1, 0xa1, 0xa1);
pub const TEXT_FAINT: Color = Color::rgb(0x70, 0x70, 0x70);

pub const SUCCESS: Color = Color::rgb(0x50, 0xe3, 0xc2);
pub const WARNING: Color = Color::rgb(0xf5, 0xa6, 0x23);
pub const DANGER: Color = Color::rgb(0xff, 0x4d, 0x4d);
pub const DANGER_SOFT: Color = Color::rgba(0xff, 0x4d, 0x4d, 36);

pub const BRAND_GRADIENT: &str = "linear-gradient(135deg, #fffdf8, #f0e6d2)";
pub const PLACEHOLDER_GRADIENT: &str = "linear-gradient(135deg, #171717, #0a0a0a)";
pub const HERO_GRADIENT: &str = "linear-gradient(120deg, #111111e6, #000000cc)";

pub fn app_theme() -> Theme {
    let mut theme = Theme::dark();
    theme.colors = ColorScheme {
        surface: SURFACE,
        surface_elevated: CARD,
        surface_hover: CARD_HOVER,
        accent: ACCENT,
        accent_hover: ACCENT_HOVER,
        accent_pressed: ACCENT_PRESSED,
        selection_background: ACCENT_SOFT,
        selection_text: TEXT,
        text_primary: TEXT,
        text_secondary: TEXT_MUTED,
        text_disabled: TEXT_FAINT,
        border: BORDER,
        border_strong: BORDER_STRONG,
        danger: DANGER,
        warning: WARNING,
        success: SUCCESS,
    };
    theme
}

pub fn row(gap: f32) -> Style {
    Style::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(AlignItems::Center)
        .gap(gap)
}

pub fn column(gap: f32) -> Style {
    Style::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(gap)
}

pub fn centered_column(gap: f32) -> Style {
    column(gap)
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
}

/// Shortens `text` to roughly fit `max_chars`, adding an ellipsis.
pub fn ellipsize(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    let kept: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{}…", kept.trim_end())
}

/// JSX children must be widgets; this renders `None` as an empty node.
pub fn optional(widget: Option<BoxedWidget>) -> BoxedWidget {
    widget.unwrap_or_else(|| Box::new(RawView::new(Style::new())))
}
