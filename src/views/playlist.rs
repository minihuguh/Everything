use dioxus::prelude::*;

#[component]
pub fn Playlist() -> Element {
    rsx! {
        div { class: "playlist-list",
            h1 { "Playlist" }
        }
    }
}