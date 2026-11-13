use crate::app::AppState;
use crate::engine;
use crate::model::{FitMode, Lang, MonitorMode, Settings};
use crate::tr;
use crate::ui::components::*;
use crate::ui::icons::AppIcon;
use crate::ui::theme::column;
use creamui::{component, jsx, BoxedWidget};
use std::rc::Rc;

fn toggle(state: &AppState, checked: bool, apply: fn(&mut Settings, bool)) -> BoxedWidget {
    let state = state.clone();
    jsx! { <Toggle checked={checked} on_toggle={callback(move || state.update_settings(|s| apply(s, !checked)))} /> }
}

#[component]
pub fn SettingsPage(state: AppState) -> BoxedWidget {
    let settings = state.settings.get();

    let fit_index = settings.fit.index();
    let fit = {
        let state = state.clone();
        jsx! {
            <Segmented selected={fit_index} options={FitMode::ALL.iter().map(|mode| mode.label()).collect()}
                on_change={Rc::new(move |index: usize| state.update_settings(|s| s.fit = FitMode::from_index(index)))} />
        }
    };

    let monitor_index = settings.monitors.index();
    let monitors = {
        let state = state.clone();
        jsx! {
            <Segmented selected={monitor_index} options={MonitorMode::ALL.iter().map(|mode| mode.label()).collect()}
                on_change={Rc::new(move |index: usize| state.update_settings(|s| s.monitors = MonitorMode::from_index(index)))} />
        }
    };

    let volume = {
        let state = state.clone();
        jsx! {
            <VolumeSlider value={settings.volume} disabled={settings.muted}
                on_change={Rc::new(move |value: f32| state.update_settings(|s| s.volume = (value * 100.0).round() / 100.0))} />
        }
    };

    let lang = {
        let state = state.clone();
        jsx! {
            <DropdownSelect
                options={Lang::ALL.iter().map(|lang| lang.label().to_owned()).collect()}
                controller={state.lang_select.clone()}
                on_change={Rc::new(move |index: usize| state.update_settings(|s| s.lang = Lang::from_index(index)))}
            />
        }
    };

    let autostart_hint = if engine::SUPPORTED {
        tr!("setting.autostart.desc")
    } else {
        tr!("setting.autostart.unsupported")
    };

    Box::new(jsx! {
        <RawView style={column(28.0).width("100%").max_width(860.0)}>
            <PageHeader title={tr!("page.settings.title")} subtitle={tr!("page.settings.subtitle")} actions={Vec::new()} />
            <SettingsSection title={tr!("section.playback")}>
                <SettingRow icon_name={AppIcon::Monitor} title={tr!("setting.fit.title")} description={tr!("setting.fit.desc")} control={fit} />
                <SettingRow icon_name={AppIcon::Monitor} title={tr!("setting.monitors.title")} description={tr!("setting.monitors.desc")} control={monitors} />
                <SettingRow icon_name={AppIcon::Volume} title={tr!("setting.mute.title")} description={tr!("setting.mute.desc")}
                    control={toggle(&state, settings.muted, |s, v| s.muted = v)} />
                <SettingRow icon_name={AppIcon::Volume} title={tr!("setting.volume.title")} description={format!("{}%", (settings.volume * 100.0).round())} control={volume} />
            </SettingsSection>
            <SettingsSection title={tr!("section.resources")}>
                <SettingRow icon_name={AppIcon::Zap} title={tr!("setting.fullscreen.title")} description={tr!("setting.fullscreen.desc")}
                    control={toggle(&state, settings.pause_on_fullscreen, |s, v| s.pause_on_fullscreen = v)} />
                <SettingRow icon_name={AppIcon::Zap} title={tr!("setting.maximized.title")} description={tr!("setting.maximized.desc")}
                    control={toggle(&state, settings.pause_on_maximized, |s, v| s.pause_on_maximized = v)} />
                <SettingRow icon_name={AppIcon::Battery} title={tr!("setting.battery.title")} description={tr!("setting.battery.desc")}
                    control={toggle(&state, settings.pause_on_battery, |s, v| s.pause_on_battery = v)} />
            </SettingsSection>
            <SettingsSection title={tr!("section.system")}>
                <SettingRow icon_name={AppIcon::Lang} title={tr!("setting.lang.title")} description={tr!("setting.lang.desc")} control={lang} />
                <SettingRow icon_name={AppIcon::Power} title={tr!("setting.autostart.title")} description={autostart_hint}
                    control={toggle(&state, settings.start_with_windows, |s, v| s.start_with_windows = v)} />
                <SettingRow icon_name={AppIcon::Film} title={tr!("setting.restore.title")} description={tr!("setting.restore.desc")}
                    control={toggle(&state, settings.restore_on_launch, |s, v| s.restore_on_launch = v)} />
                <SettingRow icon_name={AppIcon::Close} title={tr!("setting.tray.title")} description={tr!("setting.tray.desc")}
                    control={toggle(&state, settings.close_to_tray, |s, v| s.close_to_tray = v)} />
            </SettingsSection>
        </RawView>
    })
}
