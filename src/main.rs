#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
// `jsx!` expands component props as `Props { name: value }`.
#![allow(clippy::redundant_field_names)]

mod app;
mod engine;
mod model;
mod ui;

fn main() {
    app::launch();
}
