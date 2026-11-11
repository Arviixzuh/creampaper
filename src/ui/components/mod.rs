//! Reusable building blocks. Each component is a `#[component]` function so
//! pages compose them as JSX tags.

mod buttons;
mod controls;
mod empty_state;
mod nav_item;
mod notice_banner;
mod now_playing;
mod page_header;
mod scroll;
mod search_box;
mod setting_row;
mod settings_section;
mod sidebar;
mod status_badge;
mod thumbnail;
mod wallpaper_card;

pub use buttons::*;
pub use controls::*;
pub use empty_state::*;
pub use nav_item::*;
pub use notice_banner::*;
pub use now_playing::*;
pub use page_header::*;
pub use scroll::*;
pub use search_box::*;
pub use setting_row::*;
pub use settings_section::*;
pub use sidebar::*;
pub use status_badge::*;
pub use thumbnail::*;
pub use wallpaper_card::*;

/// Click handler shared between a component and the closures it builds.
pub type Callback = std::rc::Rc<dyn Fn()>;

pub fn callback(f: impl Fn() + 'static) -> Callback {
    std::rc::Rc::new(f)
}
