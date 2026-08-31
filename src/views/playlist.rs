use crate::ipc::{commands, log};
use crate::playback::reproducir_archivo_global;
use crate::state::use_player_state;
use crate::state::TrackMetadata;
use dioxus::prelude::*;
use serde::Serialize;
#[derive(Clone, PartialEq)]
pub struct Playlist {
    pub id: usize,
    pub title: &'static str,
    pub author: &'static str,
    pub song_count: usize,
    pub duration: &'static str,
    pub cover_url: Option<&'static str>,
    pub is_downloaded: bool,
}

#[derive(Clone, PartialEq)]
pub struct PlaylistTrack {
    pub number: usize,
    pub title: &'static str,
    pub artist: &'static str,
    pub duration: &'static str,
    pub cover_url: Option<&'static str>,
}

#[derive(Clone, PartialEq)]
pub struct SuggestedSong {
    pub title: &'static str,
    pub artist: &'static str,
    pub cover_url: Option<&'static str>,
}

pub fn playlists_data() -> Vec<Playlist> {
    vec![
        Playlist {
            id: 1,
            title: "Vibes nocturnas",
            author: "Tu",
            song_count: 24,
            duration: "1h 42m",
            cover_url: None,
            is_downloaded: true,
        },
        Playlist {
            id: 2,
            title: "Gym Power",
            author: "Tu",
            song_count: 56,
            duration: "3h 10m",
            cover_url: None,
            is_downloaded: false,
        },
        Playlist {
            id: 3,
            title: "Lo-Fi Study",
            author: "Tu",
            song_count: 89,
            duration: "4h 25m",
            cover_url: None,
            is_downloaded: true,
        },
        Playlist {
            id: 4,
            title: "Road Trip",
            author: "Tu",
            song_count: 32,
            duration: "2h 05m",
            cover_url: None,
            is_downloaded: false,
        },
        Playlist {
            id: 5,
            title: "Jazz Lounge",
            author: "Tu",
            song_count: 18,
            duration: "1h 20m",
            cover_url: None,
            is_downloaded: false,
        },
        Playlist {
            id: 6,
            title: "Electro 2026",
            author: "Tu",
            song_count: 45,
            duration: "2h 48m",
            cover_url: None,
            is_downloaded: true,
        },
    ]
}

pub fn playlist_tracks() -> Vec<PlaylistTrack> {
    vec![
        PlaylistTrack {
            number: 1,
            title: "Blinding Lights",
            artist: "The Weeknd",
            duration: "3:20",
            cover_url: None,
        },
        PlaylistTrack {
            number: 2,
            title: "Levitating",
            artist: "Dua Lipa",
            duration: "3:23",
            cover_url: None,
        },
        PlaylistTrack {
            number: 3,
            title: "Stay",
            artist: "The Kid LAROI",
            duration: "2:21",
            cover_url: None,
        },
        PlaylistTrack {
            number: 4,
            title: "Peaches",
            artist: "Justin Bieber",
            duration: "3:18",
            cover_url: None,
        },
        PlaylistTrack {
            number: 5,
            title: "Good 4 U",
            artist: "Olivia Rodrigo",
            duration: "2:58",
            cover_url: None,
        },
        PlaylistTrack {
            number: 6,
            title: "Montero",
            artist: "Lil Nas X",
            duration: "2:17",
            cover_url: None,
        },
        PlaylistTrack {
            number: 7,
            title: "Kiss Me More",
            artist: "Doja Cat",
            duration: "3:28",
            cover_url: None,
        },
        PlaylistTrack {
            number: 8,
            title: "Save Your Tears",
            artist: "The Weeknd",
            duration: "3:35",
            cover_url: None,
        },
    ]
}

pub fn suggested_songs() -> Vec<SuggestedSong> {
    vec![
        SuggestedSong {
            title: "After Hours",
            artist: "The Weeknd",
            cover_url: None,
        },
        SuggestedSong {
            title: "Don't Start Now",
            artist: "Dua Lipa",
            cover_url: None,
        },
        SuggestedSong {
            title: "Watermelon Sugar",
            artist: "Harry Styles",
            cover_url: None,
        },
        SuggestedSong {
            title: "Circles",
            artist: "Post Malone",
            cover_url: None,
        },
        SuggestedSong {
            title: "Heat Waves",
            artist: "Glass Animals",
            cover_url: None,
        },
    ]
}

#[component]
pub fn PlaylistView() -> Element {
    let playlists = playlists_data();
    let mut view_state = use_signal(|| PlaylistViewState::List);
    let mut selected_playlist = use_signal(|| None::<Playlist>);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/playlist.css") }

        div { class: "playlist-container",
            match view_state() {
                PlaylistViewState::List => rsx! {
                    PlaylistList {
                        playlists,
                        on_select: move |p: Playlist| {
                            selected_playlist.set(Some(p));
                            view_state.set(PlaylistViewState::Detail);
                        },
                    }
                },
                PlaylistViewState::Detail => rsx! {
                    PlaylistDetail {
                        playlist: selected_playlist().unwrap(),
                        on_back: move || view_state.set(PlaylistViewState::List),
                    }
                },
            }
        }
    }
}

#[derive(Clone, PartialEq)]
enum PlaylistViewState {
    List,
    Detail,
}

#[derive(Serialize)]
struct RespuestaPath<'a> {
    path: &'a str,
}

#[component]
fn PlaylistList(playlists: Vec<Playlist>, on_select: EventHandler<Playlist>) -> Element {
    let mut player = use_player_state();
    rsx! {
        div { class: "playlist-header",
            div { class: "playlist-header-top",
                h1 { class: "playlist-title", "Tus listas de reproduccion" }
                button {
                    class: "create-btn",
                    onclick: move |_| {},
                    svg { view_box: "0 0 24 24",
                        path { d: "M12 5v14M5 12h14", stroke: "#ffffff", fill: "none", "stroke-width": "2", "stroke-linecap": "round" }
                    }
                    "Crear nueva lista"
                }
            }
        }

        div { class: "playlist-toolbar",
            div { class: "search-box",
                svg { view_box: "0 0 24 24",
                    circle { cx: "11", cy: "11", r: "8", stroke: "#808090", fill: "none", "stroke-width": "2" }
                    line { x1: "21", y1: "21", x2: "16.65", y2: "16.65", stroke: "#808090", fill: "none", "stroke-width": "2" }
                }
                input { class: "search-input", placeholder: "Buscar listas..." }
            }
            select { class: "filter-select",
                option { value: "all", "Todas" }
                option { value: "mine", "Creadas por mi" }
                option { value: "saved", "Guardadas" }
                option { value: "downloaded", "Descargadas" }
            }
            select { class: "sort-select",
                option { value: "recent", "Recientes" }
                option { value: "name", "Nombre" }
                option { value: "most-played", "Mas escuchadas" }
            }
        }

        div { class: "fixed-lists",
            onclick: move |_| async move {
                match commands::select_document().await {
                    Some(path) => {
                        match commands::get_metadata(&*path).await {
                            Some(p) => {
                                log(&format!("Metadata: title: {}\nArtista: {}\nDuracion: {}\nimagen: {}", p.title, p.artist, p.duration, p.image));
                                let track = TrackMetadata {
                                    title: p.title,
                                    artist: p.artist,
                                    duration_secs: p.duration,
                                    path: path.clone(),
                                    image: p.image,
                                };
                                                let was_empty = player().user_queue.is_empty() && player().current_track().is_none();

                player.with_mut(|state| {
                    state.add_to_queue(track);
                });

                if was_empty {
                    let _ = commands::play_file(&path).await;
                }
                        },
                            None => (),
                        }
                    },
                    None => web_sys::console::log_1(&"select_document: cancelled".into()),
                }
            },
            div {
                class: "fixed-card",
                div { class: "fixed-icon",
                    svg { view_box: "0 0 24 24",
                        path { d: "M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z", stroke: "#7c3aed", fill: "none", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                    }
                }
                div { class: "fixed-info",
                    span { class: "fixed-title", "Tus Me Gusta" }
                    span { class: "fixed-count", "247 canciones" }
                }
            }
            div {
                class: "fixed-card",
                onclick: move |_| {},
                div { class: "fixed-icon",
                    svg { view_box: "0 0 24 24",
                        path { d: "M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z", stroke: "#7c3aed", fill: "none", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                    }
                }
                div { class: "fixed-info",
                    span { class: "fixed-title", "Archivos Locales" }
                    span { class: "fixed-count", "Importar canciones" }
                }
            }
        }

        span { class: "playlist-section-title", "Mis listas" }

        div { class: "playlist-grid",
            for (idx, playlist) in playlists.iter().enumerate() {
                PlaylistCard {
                    key: "{idx}",
                    playlist: playlist.clone(),
                    on_click: on_select.clone(),
                }
            }
        }
    }
}

#[component]
fn PlaylistCard(playlist: Playlist, on_click: EventHandler<Playlist>) -> Element {
    let p = playlist.clone();
    rsx! {
        div {
            class: "playlist-card",
            onclick: move |_| on_click.call(p.clone()),
            div { class: "playlist-card-cover",
                if let Some(url) = playlist.cover_url {
                    img { src: "{url}", alt: "{playlist.title}" }
                } else {
                    div { class: "playlist-card-placeholder",
                        svg { view_box: "0 0 24 24",
                            path { d: "M9 18V5l12-2v13", stroke: "#7c3aed", fill: "none", "stroke-width": "1.5", "stroke-linecap": "round", "stroke-linejoin": "round" }
                            circle { cx: "6", cy: "18", r: "3", stroke: "#7c3aed", fill: "none", "stroke-width": "1.5" }
                            circle { cx: "18", cy: "16", r: "3", stroke: "#7c3aed", fill: "none", "stroke-width": "1.5" }
                        }
                    }
                }
                div { class: "playlist-card-overlay",
                    svg { view_box: "0 0 24 24",
                        polygon { points: "5 3 19 12 5 21 5 3", stroke: "#ffffff", fill: "#ffffff", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                    }
                }
                if playlist.is_downloaded {
                    div { class: "playlist-downloaded",
                        svg { view_box: "0 0 24 24",
                            path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4", stroke: "#ffffff", fill: "none", "stroke-width": "3", "stroke-linecap": "round", "stroke-linejoin": "round" }
                            polyline { points: "7 10 12 15 17 10", stroke: "#ffffff", fill: "none", "stroke-width": "3", "stroke-linecap": "round", "stroke-linejoin": "round" }
                            line { x1: "12", y1: "15", x2: "12", y2: "3", stroke: "#ffffff", fill: "none", "stroke-width": "3", "stroke-linecap": "round", "stroke-linejoin": "round" }
                        }
                    }
                }
            }
            div { class: "playlist-card-info",
                span { class: "playlist-card-title", "{playlist.title}" }
                span { class: "playlist-card-meta", "{playlist.song_count} canciones / {playlist.duration}" }
            }
        }
    }
}

#[component]
fn PlaylistDetail(playlist: Playlist, on_back: EventHandler<()>) -> Element {
    let tracks = playlist_tracks();
    let suggestions = suggested_songs();

    rsx! {
        button {
            class: "back-btn",
            onclick: move |_| on_back.call(()),
            svg { view_box: "0 0 24 24",
                path { d: "M19 12H5", stroke: "currentColor", fill: "none", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                polyline { points: "12 19 5 12 12 5", stroke: "currentColor", fill: "none", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
            }
            "Volver"
        }

        div { class: "detail-header",
            div { class: "detail-cover",
                if let Some(url) = playlist.cover_url {
                    img { src: "{url}", alt: "{playlist.title}" }
                } else {
                    div { class: "detail-cover-placeholder",
                        svg { view_box: "0 0 24 24",
                            path { d: "M9 18V5l12-2v13", stroke: "#7c3aed", fill: "none", "stroke-width": "1.5", "stroke-linecap": "round", "stroke-linejoin": "round" }
                            circle { cx: "6", cy: "18", r: "3", stroke: "#7c3aed", fill: "none", "stroke-width": "1.5" }
                            circle { cx: "18", cy: "16", r: "3", stroke: "#7c3aed", fill: "none", "stroke-width": "1.5" }
                        }
                    }
                }
            }
            div { class: "detail-info",
                span { class: "detail-type", "Lista de reproduccion" }
                h1 { class: "detail-name", "{playlist.title}" }
                span { class: "detail-author", "De {playlist.author}" }
                span { class: "detail-stats", "{playlist.song_count} canciones / {playlist.duration}" }
                div { class: "detail-actions",
                    button {
                        class: "detail-play-btn",
                        onclick: move |_| {},
                        svg { view_box: "0 0 24 24",
                            polygon { points: "5 3 19 12 5 21 5 3", stroke: "#ffffff", fill: "#ffffff", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                        }
                        "Reproducir"
                    }
                    button {
                        class: "detail-shuffle-btn",
                        onclick: move |_| {},
                        svg { view_box: "0 0 24 24",
                            polyline { points: "16 3 21 3 21 8", stroke: "currentColor", fill: "none", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                            line { x1: "4", y1: "20", x2: "21", y2: "3", stroke: "currentColor", fill: "none", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                            polyline { points: "21 16 21 21 16 21", stroke: "currentColor", fill: "none", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                            line { x1: "15", y1: "15", x2: "21", y2: "21", stroke: "currentColor", fill: "none", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                            line { x1: "4", y1: "4", x2: "9", y2: "9", stroke: "currentColor", fill: "none", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                        }
                        "Aleatorio"
                    }
                    button {
                        class: "detail-menu-btn",
                        onclick: move |_| {},
                        svg { view_box: "0 0 24 24",
                            circle { cx: "12", cy: "12", r: "1", stroke: "currentColor", fill: "currentColor" }
                            circle { cx: "19", cy: "12", r: "1", stroke: "currentColor", fill: "currentColor" }
                            circle { cx: "5", cy: "12", r: "1", stroke: "currentColor", fill: "currentColor" }
                        }
                    }
                }
            }
        }

        div { class: "detail-search",
            div { class: "detail-search-box",
                svg { view_box: "0 0 24 24",
                    circle { cx: "11", cy: "11", r: "8", stroke: "#808090", fill: "none", "stroke-width": "2" }
                    line { x1: "21", y1: "21", x2: "16.65", y2: "16.65", stroke: "#808090", fill: "none", "stroke-width": "2" }
                }
                input { class: "detail-search-input", placeholder: "Buscar en esta lista..." }
            }
            button {
                class: "add-songs-btn",
                onclick: move |_| {},
                svg { view_box: "0 0 24 24",
                    path { d: "M12 5v14M5 12h14", stroke: "currentColor", fill: "none", "stroke-width": "2", "stroke-linecap": "round" }
                }
                "Anadir canciones"
            }
        }

        div { class: "track-list",
            div { class: "track-header",
                span { "#" }
                span { "Titulo" }
                span { "Duracion" }
                span {}
            }
            for (idx, track) in tracks.iter().enumerate() {
                TrackRow {
                    key: "{idx}",
                    number: track.number,
                    title: track.title,
                    artist: track.artist,
                    duration: track.duration,
                    cover_url: track.cover_url,
                }
            }
        }

        div { class: "suggestions-section",
            span { class: "suggestions-title", "Canciones recomendadas" }
            div { class: "suggestions-row",
                for (idx, song) in suggestions.iter().enumerate() {
                    SuggestionCard {
                        key: "{idx}",
                        title: song.title,
                        artist: song.artist,
                        cover_url: song.cover_url,
                    }
                }
            }
        }
    }
}

#[component]
fn TrackRow(
    number: usize,
    title: &'static str,
    artist: &'static str,
    duration: &'static str,
    cover_url: Option<&'static str>,
) -> Element {
    rsx! {
        div { class: "track-item",
            div { class: "track-number-wrap",
                span { class: "track-number", "{number}" }
                div { class: "track-play-icon",
                    svg { view_box: "0 0 24 24",
                        polygon { points: "5 3 19 12 5 21 5 3", stroke: "#ffffff", fill: "#ffffff", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                    }
                }
            }
            div { class: "track-info",
                div { class: "track-cover",
                    if let Some(url) = cover_url {
                        img { src: "{url}", alt: "{title}" }
                    } else {
                        svg { view_box: "0 0 24 24",
                            path { d: "M9 18V5l12-2v13", stroke: "#7c3aed", fill: "none", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                            circle { cx: "6", cy: "18", r: "3", stroke: "#7c3aed", fill: "none", "stroke-width": "2" }
                            circle { cx: "18", cy: "16", r: "3", stroke: "#7c3aed", fill: "none", "stroke-width": "2" }
                        }
                    }
                }
                div { class: "track-text",
                    span { class: "track-title", "{title}" }
                    span { class: "track-artist", "{artist}" }
                }
            }
            span { class: "track-duration", "{duration}" }
            button {
                class: "track-menu-btn",
                onclick: move |_| {},
                svg { view_box: "0 0 24 24",
                    circle { cx: "12", cy: "12", r: "1", stroke: "currentColor", fill: "currentColor" }
                    circle { cx: "19", cy: "12", r: "1", stroke: "currentColor", fill: "currentColor" }
                    circle { cx: "5", cy: "12", r: "1", stroke: "currentColor", fill: "currentColor" }
                }
            }
        }
    }
}

#[component]
fn SuggestionCard(
    title: &'static str,
    artist: &'static str,
    cover_url: Option<&'static str>,
) -> Element {
    rsx! {
        div { class: "suggestion-card",
            div { class: "suggestion-cover",
                if let Some(url) = cover_url {
                    img { src: "{url}", alt: "{title}" }
                } else {
                    div { class: "suggestion-cover-placeholder",
                        svg { view_box: "0 0 24 24",
                            path { d: "M9 18V5l12-2v13", stroke: "#7c3aed", fill: "none", "stroke-width": "1.5", "stroke-linecap": "round", "stroke-linejoin": "round" }
                            circle { cx: "6", cy: "18", r: "3", stroke: "#7c3aed", fill: "none", "stroke-width": "1.5" }
                            circle { cx: "18", cy: "16", r: "3", stroke: "#7c3aed", fill: "none", "stroke-width": "1.5" }
                        }
                    }
                }
                div { class: "suggestion-add",
                    svg { view_box: "0 0 24 24",
                        path { d: "M12 5v14M5 12h14", stroke: "#ffffff", fill: "none", "stroke-width": "2", "stroke-linecap": "round" }
                    }
                }
            }
            div { class: "suggestion-info",
                span { class: "suggestion-title", "{title}" }
                span { class: "suggestion-artist", "{artist}" }
            }
        }
    }
}
