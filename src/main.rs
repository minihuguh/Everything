mod app;
mod components;
mod ipc;
mod playback;
mod state;
mod util;
mod views;


use app::App;
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}
