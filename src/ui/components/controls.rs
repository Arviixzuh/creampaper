use super::*;
use creamui::widgets::{SegmentedControl, Switch};
use creamui::{component, jsx, BoxedWidget};
use std::rc::Rc;

#[component]
pub fn Toggle(checked: bool, on_toggle: Callback) -> BoxedWidget {
    Box::new(Switch::new(checked, move || on_toggle()))
}

#[component]
pub fn Segmented(
    selected: usize,
    options: Vec<&'static str>,
    on_change: Rc<dyn Fn(usize)>,
) -> BoxedWidget {
    let mut control = SegmentedControl::new(selected, move |index| on_change(index));
    for option in options {
        control = control.option(option);
    }
    Box::new(control)
}

#[component]
pub fn VolumeSlider(value: f32, disabled: bool, on_change: Rc<dyn Fn(f32)>) -> BoxedWidget {
    if disabled {
        return Box::new(jsx! {
            <RawText color={crate::ui::theme::TEXT_FAINT} font_size={12.5}>"Silenciado"</RawText>
        });
    }
    Box::new(jsx! {
        <Slider value={value} on_change={move |next: f32| on_change(next)} width={180.0} height={20.0} />
    })
}
