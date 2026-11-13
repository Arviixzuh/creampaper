use crate::app::AppState;
use crate::tr;
use crate::ui::components::*;
use crate::ui::icons::AppIcon;
use crate::ui::theme::{self, column, row};
use creamui::core::layout::AlignItems;
use creamui::widgets::RawView;
use creamui::{component, jsx, BoxedWidget};

const GRID_GAP: f32 = 16.0;
const MIN_CARD_WIDTH: f32 = 220.0;

/// Splits the available width into equally sized columns.
fn grid_columns(content_width: f32) -> (usize, f32) {
    let columns = ((content_width + GRID_GAP) / (MIN_CARD_WIDTH + GRID_GAP))
        .floor()
        .max(1.0) as usize;
    let card_width = (content_width - GRID_GAP * (columns as f32 - 1.0)) / columns as f32;
    (columns, card_width.floor())
}

#[component]
pub fn LibraryPage(state: AppState, content_width: f32) -> BoxedWidget {
    let _language = state.settings.get().lang;
    let library = state.library.get();
    let query = state.search.get();
    let thumbnails = state.thumbnails.get();
    let active = state.active.get();
    let (columns, card_width) = grid_columns(content_width);

    let visible: Vec<_> = library
        .iter()
        .filter(|item| item.matches(&query))
        .cloned()
        .collect();
    let add = state.clone();
    let header_actions: Vec<BoxedWidget> = vec![
        jsx! { <SearchBox query={state.search.clone()} width={240.0} /> },
        jsx! { <PrimaryButton icon_name={Some(AppIcon::Plus)} label={tr!("library.add")} on_click={callback(move || add.pick_files())} /> },
    ];

    let body: BoxedWidget = if library.is_empty() {
        let add = state.clone();
        jsx! {
            <EmptyState title={tr!("library.empty.title")}
                message={tr!("library.empty.message")}
                on_add={Some(callback(move || add.pick_files()))} />
        }
    } else if visible.is_empty() {
        let clear = state.clone();
        Box::new(jsx! {
            <RawView style={column(14.0).width("100%")}>
                <EmptyState title={tr!("library.no_results.title")} message={tr!("library.no_results.message", query=query)} on_add={None} />
                <RawView style={row(0.0)}>
                    <GhostButton icon_name={Some(AppIcon::Close)} label={tr!("library.clear_search")} on_click={callback(move || clear.search.set(String::new()))} />
                </RawView>
            </RawView>
        })
    } else {
        let cards: Vec<BoxedWidget> = visible
            .into_iter()
            .map(|wallpaper| {
                let id = wallpaper.id;
                let apply = state.clone();
                let remove = state.clone();
                jsx! {
                    <WallpaperCard thumbnail={thumbnails.get(&id).cloned()} wallpaper={wallpaper}
                        width={card_width} active={active == Some(id)}
                        on_apply={callback(move || apply.apply(id))}
                        on_remove={callback(move || remove.remove(id))} />
                }
            })
            .collect();
        let mut cards = cards.into_iter();
        let rows: Vec<BoxedWidget> = std::iter::from_fn(|| {
            let chunk: Vec<BoxedWidget> = cards.by_ref().take(columns).collect();
            (!chunk.is_empty()).then(|| -> BoxedWidget {
                Box::new(
                    RawView::new(row(GRID_GAP).align_items(AlignItems::FlexStart))
                        .with_children(chunk),
                )
            })
        })
        .collect();
        Box::new(RawView::new(column(GRID_GAP).width("100%")).with_children(rows))
    };

    let count = library.len();
    let subtitle = if count == 1 {
        tr!("library.subtitle.one", count = count)
    } else {
        tr!("library.subtitle.many", count = count)
    };
    Box::new(jsx! {
        <RawView style={column(24.0).width("100%")}>
            <PageHeader title={tr!("library.title")}
                subtitle={subtitle}
                actions={header_actions} />
            <NowPlaying state={state.clone()} />
            <RawText color={theme::TEXT_FAINT} font_size={11.5} bold={true}>{tr!("library.wallpapers")}</RawText>
            {body}
        </RawView>
    })
}
