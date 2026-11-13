use creamui::widgets::{Select, SelectController};
use creamui::{component, jsx, BoxedWidget};
use std::rc::Rc;

#[component]
fn SelectControl(
    options: Vec<String>,
    controller: SelectController,
    on_change: Rc<dyn Fn(usize)>,
) -> BoxedWidget {
    let options: Vec<&str> = options.iter().map(String::as_str).collect();
    Box::new(Select::controlled(&options, controller).on_select(move |index| on_change(index)))
}

#[component]
pub fn DropdownSelect(
    options: Vec<String>,
    controller: SelectController,
    on_change: Rc<dyn Fn(usize)>,
) -> BoxedWidget {
    jsx! {
        <SelectControl options={options} controller={controller} on_change={on_change} />
    }
}
