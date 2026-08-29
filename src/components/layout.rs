use dioxus::prelude::*;
use crate::components::{sidebar::Sidebar, player_bar::PlayerBar, content_header::ContentHeader};
use crate::state::{use_player_state, provide_player_state};
use crate::views::{home::Home, search::Search, lyrics::Lyrics, playlist::PlaylistView, stadistics::Stadistics, queue::QueuePanel};

#[component]
pub fn Layout() -> Element {
    // Proveer el estado global una sola vez en el layout raíz
    provide_player_state();

    let player = use_player_state();
    let current_view = use_memo(move || player().current_view);
    let mut show_queue = use_signal(|| false);

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
                            "playlist" => rsx! { PlaylistView {} },
                            "stadistics" => rsx! { Stadistics {} },
                            _ => rsx! { Home {} },
                        }
                    }
                }
            }

            PlayerBar { show_queue}
            QueuePanel { show: show_queue }
            div {
                class: if show_queue() { "queue-backdrop open" } else { "queue-backdrop" },
                onclick: move |_| show_queue.set(false),
            }
        }
    }
}