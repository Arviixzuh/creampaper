use crate::ui::icons::{icon, AppIcon};
use crate::ui::theme::{self, row};
use creamui::{component, jsx, BoxedWidget, Signal};

#[component]
pub fn SearchBox(query: Signal<String>, width: f32) -> BoxedWidget {
    let value = query.get();
    let style = row(8.0)
        .width(width)
        .height(38.0)
        .padding("0px 0px 0px 12px")
        .background(theme::FIELD)
        .border(theme::BORDER, 1.0)
        .corner_radius(10.0);
    Box::new(jsx! {
        <RawView style={style}>
            {icon(AppIcon::Search, theme::TEXT_FAINT, 16.0)}
            <TextInput value={value} on_change={move |next: String| query.set(next)}
                placeholder={"Buscar wallpapers…"} width={width - 40.0} height={34.0}
                background={creamui::Color::rgba(0, 0, 0, 0)} border={(creamui::Color::rgba(0, 0, 0, 0), 0.0)} />
        </RawView>
    })
}
