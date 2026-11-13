use crate::tr;
use crate::ui::components::*;
use crate::ui::icons::AppIcon;
use crate::ui::theme::{self, column};
use creamui::{component, jsx, BoxedWidget};

fn note(text: &str) -> BoxedWidget {
    Box::new(
        jsx! { <RawText color={theme::TEXT_MUTED} font_size={12.5}>{text.to_owned()}</RawText> },
    )
}

#[component]
pub fn AboutPage(version: String) -> BoxedWidget {
    Box::new(jsx! {
        <RawView style={column(28.0).width("100%").max_width(860.0)}>
            <PageHeader title={tr!("about.title")} subtitle={tr!("about.subtitle", version=version)} actions={Vec::new()} />
            <SettingsSection title={tr!("about.how_it_works")}>
                <SettingRow icon_name={AppIcon::Film} title={tr!("about.media_foundation.title")} description={tr!("about.media_foundation.desc")} control={note("GPU")} />
                <SettingRow icon_name={AppIcon::Monitor} title={tr!("about.desktop_layer.title")} description={tr!("about.desktop_layer.desc")} control={note("WorkerW")} />
                <SettingRow icon_name={AppIcon::Zap} title={tr!("about.low_power.title")} description={tr!("about.low_power.desc")} control={note(&tr!("about.auto_pause"))} />
            </SettingsSection>
            <SettingsSection title={tr!("about.shortcuts")}>
                <SettingRow icon_name={AppIcon::Plus} title={tr!("about.drag_drop.title")} description={tr!("about.drag_drop.desc")} control={note(&tr!("about.drag_drop.label"))} />
                <SettingRow icon_name={AppIcon::Power} title={tr!("about.tray.title")} description={tr!("about.tray.desc")} control={note(&tr!("about.tray.label"))} />
            </SettingsSection>
        </RawView>
    })
}
