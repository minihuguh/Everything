use crate::ipc::commands;
use crate::state::{use_player_state, QueueSource, TrackMetadata};
use dioxus::prelude::*;

#[component]
pub fn QueuePanel(show: Signal<bool>) -> Element {
    let mut player = use_player_state();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/queue.css") }
        div {
            class: if show() { "queue-panel open" } else { "queue-panel" },
            div { class: "queue-header",
                span { class: "queue-title", "Cola de reproduccion" }
                button {
                    class: "queue-close-btn",
                    onclick: move |_| show.set(false),
                    svg { view_box: "0 0 24 24",
                        path { d: "M18 6L6 18M6 6l12 12", stroke: "#808090", fill: "none", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                    }
                }
            }

            if let Some(current) = player().current_track() {
                div { class: "queue-now-playing",
                    span { class: "queue-section-label", "Reproduciendo ahora" }
                    QueueItem {
                        track: current.clone(),
                        is_active: true,
                        is_past: false,
                        show_number: false,
                        number: 0,
                        on_click: None,
                        on_remove: None,
                    }
                }
            }

            div { class: "queue-body",
                div { class: "queue-section",
                    span { class: "queue-section-label", "Cola" }
                    div { class: "queue-list",
                        QueueUserQueueList { player }
                    }
                }

                div { class: "queue-section",
                    span { class: "queue-section-label",
                        {
                            let name = player().active_playlist_name.clone();
                            if name.is_empty() {
                                "Lista de reproduccion".to_string()
                            } else {
                                format!("Desde: {}", name)
                            }
                        }
                    }
                    div { class: "queue-list",
                        QueueActivePlaylistList { player }
                    }
                }
            }

            div { class: "queue-footer",
                if player().user_queue_len() > 0 {
                    button {
                        class: "queue-clear-btn",
                        onclick: move |_| player.write().clear_user_queue(),
                        svg { view_box: "0 0 24 24",
                            path { d: "M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2", stroke: "currentColor", fill: "none", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                        }
                        "Limpiar cola"
                    }
                }
            }
        }
    }
}

#[component]
fn QueueUserQueueList(player: Signal<crate::state::PlayerState>) -> Element {
    let user_queue = player().user_queue.clone();
    let current_source = player().current_source;
    let current_queue_idx = player().current_queue_index;

    rsx! {
        if user_queue.is_empty() {
            div { class: "queue-empty", "No hay canciones en la cola" }
        } else {
            for (idx, track) in user_queue.into_iter().enumerate() {
                {
                    let is_in_queue = current_source == QueueSource::Queue;
                    let is_active = is_in_queue && current_queue_idx == Some(idx);
                    let is_past = is_in_queue && current_queue_idx.map_or(false, |cur| idx < cur);

                    rsx! {
                        QueueItem {
                            key: "uq-{idx}",
                            track: track,
                            is_active: is_active,
                            is_past: is_past,
                            show_number: true,
                            number: idx + 1,
                            on_click: Some(EventHandler::new(move |_| {
                                player.write().jump_to_queue(idx);
                                if let Some(track) = player.read().current_track().cloned() {
                                spawn (async move {
                                    let _ = commands::play_file(&track.path).await;
                                });
                            }
                            })),
                            on_remove: Some(EventHandler::new(move |_| {
                                player.write().remove_from_queue(idx);
                            })),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn QueueActivePlaylistList(player: Signal<crate::state::PlayerState>) -> Element {
    let playlist = player().active_playlist.clone();
    let current_source = player().current_source;
    let current_playlist_idx = player().current_playlist_index;

    rsx! {
        if playlist.is_empty() {
            div { class: "queue-empty", "No hay mas canciones en la lista" }
        } else {
            for (idx, track) in playlist.into_iter().enumerate() {
                {
                    let is_in_playlist = current_source == QueueSource::Playlist;
                    let is_active = is_in_playlist && current_playlist_idx == Some(idx);
                    let is_past = current_playlist_idx.map_or(false, |cur| idx < cur);

                    rsx! {
                        QueueItem {
                            key: "ap-{idx}",
                            track: track,
                            is_active: is_active,
                            is_past: is_past,
                            show_number: true,
                            number: idx + 1,
                            on_click: Some(EventHandler::new(move |_| {
                                player.write().jump_to_playlist(idx);
                            })),
                            on_remove: None,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn QueueItem(
    track: TrackMetadata,
    is_active: bool,
    #[props(default = false)] is_past: bool,
    show_number: bool,
    #[props(default = 0)] number: usize,
    on_click: Option<EventHandler<()>>,
    on_remove: Option<EventHandler<()>>,
) -> Element {
    let item_class = if is_active {
        "queue-item active"
    } else if is_past {
        "queue-item past"
    } else {
        "queue-item"
    };
    rsx! {
        div {
            class: "{item_class}",
            onclick: move |_| {
                if let Some(handler) = on_click {
                    handler.call(());
                }
            },
            if show_number {
                span { class: "queue-item-number", "{number}" }
            }
            div { class: "queue-item-cover",
                if track.image.is_empty() {
                    svg { view_box: "0 0 24 24",
                        path { d: "M9 18V5l12-2v13", stroke: "#7c3aed", fill: "none", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                        circle { cx: "6", cy: "18", r: "3", stroke: "#7c3aed", fill: "none", "stroke-width": "2" }
                        circle { cx: "18", cy: "16", r: "3", stroke: "#7c3aed", fill: "none", "stroke-width": "2" }
                    }
                } else {
                    img { src: "data:image/jpeg;base64,{track.image}", alt: "{track.title}" }
                }
            }
            div { class: "queue-item-info",
                span { class: "queue-item-title", "{track.title}" }
                span { class: "queue-item-artist", "{track.artist}" }
            }
            span { class: "queue-item-duration",
                {
                    let total: u64 = track.duration_secs.round() as u64;
                    let mins: u64 = total / 60;
                    let secs: u64 = total % 60;
                    format!("{mins:02}:{secs:02}")
                }
            }
            if let Some(handler) = on_remove {
                button {
                    class: "queue-item-remove",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        handler.call(());
                    },
                    svg { view_box: "0 0 24 24",
                        path { d: "M18 6L6 18M6 6l12 12", stroke: "currentColor", fill: "none", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                    }
                }
            }
        }
    }
}