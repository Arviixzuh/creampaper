//! Presentation layer: everything here is built with `jsx!`.
//!
//! - `theme`: palette and layout helpers
//! - `icons`: cached SVG line icons
//! - `components`: reusable `#[component]`s
//! - `pages`: one view per sidebar destination

mod components;
mod icons;
mod pages;
pub mod theme;
use crate::app::{AppState, Page};
use components::*;
use creamui::{jsx, BoxedWidget, Size};
use pages::*;
use theme::{column, optional, row, PAGE_PADDING, SCROLLBAR_ALLOWANCE, SIDEBAR_WIDTH};

pub fn root(state: AppState, viewport: Size) -> BoxedWidget {
    let content_width =
        (viewport.width - SIDEBAR_WIDTH - PAGE_PADDING * 2.0 - SCROLLBAR_ALLOWANCE).max(320.0);

    let page: BoxedWidget = match state.page.get() {
        Page::Library => {
            jsx! { <LibraryPage state={state.clone()} content_width={content_width} /> }
        }
        Page::Settings => jsx! { <SettingsPage state={state.clone()} /> },
        Page::About => jsx! { <AboutPage version={env!("CARGO_PKG_VERSION").to_owned()} /> },
    };
    let notice = state.notice.get().map(|notice| -> BoxedWidget {
        let dismiss = state.clone();
        jsx! { <NoticeBanner notice={notice} on_dismiss={callback(move || dismiss.dismiss_notice())} /> }
    });

    let background = if state.repaint.get() {
        theme::BACKGROUND_REPAINT
    } else {
        theme::BACKGROUND
    };
    let content = column(20.0).width("100%").padding(format!(
        "28px {}px {PAGE_PADDING}px {PAGE_PADDING}px",
        PAGE_PADDING + SCROLLBAR_ALLOWANCE
    ));
    Box::new(jsx! {
        <RawView style={row(0.0).width(viewport.width).height(viewport.height).background(background)}>
            <Sidebar state={state.clone()} viewport_height={viewport.height} />
            <Scroll controller={state.scroll.clone()} style={creamui::Style::new().width(viewport.width - SIDEBAR_WIDTH).height(viewport.height)}>
                <RawView style={content}>
                    {optional(notice)}
                    {page}
                </RawView>
            </Scroll>
        </RawView>
    })
}
