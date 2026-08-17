use dioxus::prelude::*;
// use crate::state::provide_player_state;
use crate::components::layout::Layout;
// use crate::state::provide_app_state;

#[component]
pub fn App() -> Element {
    // provide_app_state();
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/main.css") }
        Layout {}
    }
}