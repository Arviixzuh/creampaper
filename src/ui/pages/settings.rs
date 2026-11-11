use crate::app::AppState;
use crate::engine;
use crate::model::{FitMode, MonitorMode, Settings};
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
    let autostart_hint = if engine::SUPPORTED {
        "Arranca minimizado en la bandeja al iniciar sesión"
    } else {
        "Disponible solo en Windows"
    };

    Box::new(jsx! {
        <RawView style={column(28.0).width("100%").max_width(860.0)}>
            <PageHeader title={"Ajustes".to_owned()} subtitle={"Los cambios se aplican y guardan al instante".to_owned()} actions={Vec::new()} />
            <SettingsSection title={"Reproducción".to_owned()}>
                <SettingRow icon_name={AppIcon::Monitor} title={"Escalado".to_owned()} description={"Cómo se adapta el video a la pantalla".to_owned()} control={fit} />
                <SettingRow icon_name={AppIcon::Monitor} title={"Monitores".to_owned()} description={"Dónde se muestra el fondo animado".to_owned()} control={monitors} />
                <SettingRow icon_name={AppIcon::Volume} title={"Silenciar audio".to_owned()} description={"Los fondos se reproducen sin sonido por defecto".to_owned()}
                    control={toggle(&state, settings.muted, |s, v| s.muted = v)} />
                <SettingRow icon_name={AppIcon::Volume} title={"Volumen".to_owned()} description={format!("{}%", (settings.volume * 100.0).round())} control={volume} />
            </SettingsSection>
            <SettingsSection title={"Ahorro de recursos".to_owned()}>
                <SettingRow icon_name={AppIcon::Zap} title={"Pausar con apps a pantalla completa".to_owned()} description={"Juegos y videos a pantalla completa no compiten por la GPU".to_owned()}
                    control={toggle(&state, settings.pause_on_fullscreen, |s, v| s.pause_on_fullscreen = v)} />
                <SettingRow icon_name={AppIcon::Zap} title={"Pausar con ventanas maximizadas".to_owned()} description={"Si el escritorio no se ve, no se decodifica".to_owned()}
                    control={toggle(&state, settings.pause_on_maximized, |s, v| s.pause_on_maximized = v)} />
                <SettingRow icon_name={AppIcon::Battery} title={"Pausar con batería".to_owned()} description={"Detiene la animación al desconectar el cargador".to_owned()}
                    control={toggle(&state, settings.pause_on_battery, |s, v| s.pause_on_battery = v)} />
            </SettingsSection>
            <SettingsSection title={"Sistema".to_owned()}>
                <SettingRow icon_name={AppIcon::Power} title={"Iniciar con Windows".to_owned()} description={autostart_hint.to_owned()}
                    control={toggle(&state, settings.start_with_windows, |s, v| s.start_with_windows = v)} />
                <SettingRow icon_name={AppIcon::Film} title={"Restaurar último fondo".to_owned()} description={"Vuelve a reproducir el último wallpaper al abrir".to_owned()}
                    control={toggle(&state, settings.restore_on_launch, |s, v| s.restore_on_launch = v)} />
                <SettingRow icon_name={AppIcon::Close} title={"Cerrar a la bandeja".to_owned()} description={"El fondo sigue activo al cerrar la ventana".to_owned()}
                    control={toggle(&state, settings.close_to_tray, |s, v| s.close_to_tray = v)} />
            </SettingsSection>
        </RawView>
    })
}
