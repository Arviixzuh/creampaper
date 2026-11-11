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
            <PageHeader title={"Acerca de".to_owned()} subtitle={format!("CreamPaper {version} · hecho con CreamUI")} actions={Vec::new()} />
            <SettingsSection title={"Cómo funciona".to_owned()}>
                <SettingRow icon_name={AppIcon::Film} title={"Media Foundation".to_owned()} description={"Decodificación por hardware nativa de Windows, sin navegador embebido".to_owned()} control={note("GPU")} />
                <SettingRow icon_name={AppIcon::Monitor} title={"Detrás de los iconos".to_owned()} description={"El video se dibuja en la capa WorkerW del escritorio".to_owned()} control={note("WorkerW")} />
                <SettingRow icon_name={AppIcon::Zap} title={"Bajo consumo".to_owned()} description={"La interfaz solo se redibuja al cambiar el estado; el motor se pausa cuando el escritorio no es visible".to_owned()} control={note("Auto-pausa")} />
            </SettingsSection>
            <SettingsSection title={"Atajos".to_owned()}>
                <SettingRow icon_name={AppIcon::Plus} title={"Arrastrar y soltar".to_owned()} description={"Suelta videos sobre la ventana para añadirlos".to_owned()} control={note("Drag & drop")} />
                <SettingRow icon_name={AppIcon::Power} title={"Bandeja del sistema".to_owned()} description={"Clic en el icono para abrir; menú para pausar o salir".to_owned()} control={note("Tray")} />
            </SettingsSection>
        </RawView>
    })
}
