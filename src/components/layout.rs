use dioxus::prelude::*;
use crate::components::{sidebar::Sidebar, player_bar::PlayerBar, content_header::ContentHeader};
use crate::state::use_app_state;
use crate::views::{home::Home, search::Search, lyrics::Lyrics, playlist::Playlist};

#[component]
pub fn Layout() -> Element {
    let current_view = use_app_state();

    rsx! {
        div { class: "app-layout",
            div { class: "app-main",
                Sidebar {},

                div { class: "content-wrapper",
                    ContentHeader {},

                    main { class: "content-body",
                        match current_view() {
                            "home" => rsx! { Home {} },
                            "search" => rsx! { Search {} },
                            "lyrics" => rsx! { Lyrics {} },
                            "playlist" => rsx! { Playlist {} },
                            _ => rsx! { Home {} },
                        }
                    }
                }
            }

            PlayerBar {}
        }
    }
}