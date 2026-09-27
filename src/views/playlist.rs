use crate::ipc::{commands, log};
use crate::state::TrackMetadata;
use crate::state::use_player_state;
use dioxus::prelude::*;
use shared::{SavedPlaylist, SavedTrack};
use wasm_bindgen::JsValue;

#[derive(Clone, PartialEq)]
pub struct Playlist {
    pub id: String,
    pub title: String,
    pub author: String,
    pub song_count: usize,
    pub duration: String,
    pub cover_url: Option<String>,
    pub path: String,
    pub is_downloaded: bool,
    pub tracks: Vec<TrackMetadata>,
}

// #[derive(Clone, PartialEq)]
// pub struct PlaylistTrack {
//     pub number: usize,
//     pub title: &'static str,
//     pub artist: &'static str,
//     pub duration: &'static str,
//     pub cover_url: Option<&'static str>,
// }

#[derive(Clone, PartialEq)]
pub struct SuggestedSong {
    pub title: &'static str,
    pub artist: &'static str,
    pub cover_url: Option<&'static str>,
}

// pub fn suggested_songs() -> Vec<SuggestedSong> {
//     vec![
//         SuggestedSong {
//             title: "After Hours",
//             artist: "The Weeknd",
//             cover_url: None,
//         },
//         SuggestedSong {
//             title: "Don't Start Now",
//             artist: "Dua Lipa",
//             cover_url: None,
//         },
//         SuggestedSong {
//             title: "Watermelon Sugar",
//             artist: "Harry Styles",
//             cover_url: None,
//         },
//         SuggestedSong {
//             title: "Circles",
//             artist: "Post Malone",
//             cover_url: None,
//         },
//         SuggestedSong {
//             title: "Heat Waves",
//             artist: "Glass Animals",
//             cover_url: None,
//         },
//     ]
// }

#[component]
pub fn PlaylistView() -> Element {
    let mut view_state = use_signal(|| PlaylistViewState::List);
    let mut selected_playlist = use_signal(|| None::<Playlist>);

    let mut playlists_resource = use_resource(|| async move {
        if let Some(p) = commands::get_playlists().await {
            web_sys::console::log_1(&"Imprimiendo playlist from get_playlists".into());
            let mensaje = JsValue::from_str(&format!("{:?}", p));
            web_sys::console::log_1(&mensaje);

            let mapped_playlists = p
                .into_iter()
                .map(|sp| {
                    let total_secs = sp.tracks.iter().map(|t| t.duration_secs).sum::<f64>();
                    let mins = (total_secs / 60.0) as usize;

                    Playlist {
                        id: sp.id,
                        title: sp.title,
                        author: sp.author,
                        song_count: sp.tracks.len(),
                        duration: format!("{mins}m"),
                        cover_url: sp
                            .tracks
                            .first()
                            .map(|t| t.image.clone())
                            .filter(|img| !img.is_empty()),
                        path: sp.file_path,
                        is_downloaded: true,
                        tracks: sp
                            .tracks
                            .into_iter()
                            .map(|st| TrackMetadata {
                                title: st.title,
                                artist: st.artist,
                                duration_secs: st.duration_secs,
                                path: st.path,
                                image: st.image,
                            })
                            .collect(),
                    }
                })
                .collect::<Vec<Playlist>>();

            Some(mapped_playlists)
        } else {
            None
        }
    });

    let playlists = playlists_resource
        .value()
        .read()
        .as_ref()
        .cloned()
        .flatten()
        .unwrap_or_default();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/playlist.css") }

        div { class: "playlist-container",
            match view_state() {
                PlaylistViewState::List => rsx! {
                    PlaylistList {
                        playlists,
                        on_select: move |mut p: Playlist| {
                            spawn(async move {
                                for track in p.tracks.iter_mut() {
                                    if !track.path.is_empty() {
                                        if let Some(meta) = commands::get_metadata(&track.path).await {
                                            if !meta.image.is_empty() {
                                                track.image = meta.image;
                                            }
                                        }
                                    }
                                }
                                if let Some(first) = p.tracks.first() {
                                    if !first.image.is_empty() {
                                        p.cover_url = Some(first.image.clone());
                                    }
                                }
                                selected_playlist.set(Some(p));
                                view_state.set(PlaylistViewState::Detail);
                            });
                        },
                        on_refresh: move || {
                            playlists_resource.restart();
                        }
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

#[component]
fn PlaylistList(
    playlists: Vec<Playlist>,
    on_select: EventHandler<Playlist>,
    on_refresh: EventHandler<()>,
) -> Element {
    let mut player = use_player_state();
    rsx! {
            div { class: "playlist-header",
                div { class: "playlist-header-top",
                    h1 { class: "playlist-title", "Tus listas de reproduccion" }
                    button {
                        class: "create-btn",
                        onclick: move |_| {
                            let refresh_trigger = on_refresh.clone();
                            spawn(async move {
                                let new_playlist = SavedPlaylist {
                                    id: format!("pl_{}", js_sys::Date::now() as u64),
                                    title: "Nueva Lista".to_string(),
                                    author: "Usuario".to_string(),
                                    file_path: "".to_string(),
                                    tracks: vec![],
                                    total_duration_secs: 0.0,
                                };

                                if let Ok(_) = commands::save_playlist_ipc(new_playlist).await {
                                    log("Nueva playlist creada.");
                                    refresh_trigger.call(());
                                }
                            });
                        },
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
    onclick: move |_| {
        let refresh_trigger = on_refresh.clone();
        spawn(async move {
            let was_empty = player.read().active_playlist.is_empty() && player.read().current_track().is_none();
            if let Some(pl) = commands::playlist().await {
                let playlist_name = pl
                    .title
                    .clone()
                    .unwrap_or_else(|| "Lista sin nombre".to_string());

                let mut loaded_tracks = Vec::new();
                let mut saved_tracks = Vec::new();

                for track in pl.track_list.tracks {
                    let path = track.location.clone().unwrap_or_default().replace("file:///", "");
                    let mut title = track.title.clone().unwrap_or_else(|| "Sin título".to_string());
                    let mut artist = track.creator.clone().unwrap_or_else(|| "Artista desconocido".to_string());
                    let mut duration_secs = track.duration.map(|ms| ms as f64 / 1000.0).unwrap_or(0.0);
                    let mut image = String::new();

                    if !path.is_empty() {
                        if let Some(p) = commands::get_metadata(&path).await {
                            image = p.image;
                            if title == "Sin título" && !p.title.is_empty() { title = p.title; }
                            if artist == "Artista desconocido" && !p.artist.is_empty() { artist = p.artist; }
                            if duration_secs == 0.0 && p.duration > 0.0 { duration_secs = p.duration; }
                        }
                    }

                    loaded_tracks.push(TrackMetadata {
                        title: title.clone(),
                        artist: artist.clone(),
                        duration_secs,
                        path: path.clone(),
                        image,
                    });

                    saved_tracks.push(SavedTrack {
                        title,
                        artist,
                        duration_secs,
                        path,
                        image: String::new(),
                    });
                }

                if loaded_tracks.is_empty() {
                    return;
                }

                let saved_playlist = SavedPlaylist {
                    id: format!("pl_{}", js_sys::Date::now() as u64),
                    title: playlist_name.clone(),
                    author: "Local".to_string(),
                    file_path: "".to_string(),
                    tracks: saved_tracks,
                    total_duration_secs: loaded_tracks.iter().map(|t| t.duration_secs).sum(),
                };

                match commands::save_playlist_ipc(saved_playlist).await {
                    Ok(_) => {
                        log("Playlist guardada con éxito.");
                        refresh_trigger.call(());
                    }
                    Err(e) => {
                        log(&format!("Error guardando playlist: {e}"));
                    }
                }

                let first_track_path = loaded_tracks[0].path.clone();
                player.write().play_playlist(loaded_tracks, playlist_name, 0);

                if was_empty && !first_track_path.is_empty() {
                    let _ = commands::play_file(&first_track_path).await;
                }
            }
        });
    },
                    class: "fixed-card",
                    div { class: "fixed-icon",
                        svg { view_box: "0 0 24 24",
                            path { d: "M2 5C2 4.05719 2 3.58579 2.29289 3.29289C2.58579 3 3.05719 3 4 3H20C20.9428 3 21.4142 3 21.7071 3.29289C22 3.58579 22 4.05719 22 5C22 5.94281 22 6.41421 21.7071 6.70711C21.4142 7 20.9428 7 20 7H4C3.05719 7 2.58579 7 2.29289 6.70711C2 6.41421 2 5.94281 2 5Z", stroke: "#7c3aed", fill: "#1C274C", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                            path { d: "M20.0689 8.49993C20.2101 8.49999 20.3551 8.50005 20.5 8.49805V12.9999C20.5 16.7711 20.5 18.6568 19.3284 19.8283C18.1569 20.9999 16.2712 20.9999 12.5 20.9999H11.5C7.72876 20.9999 5.84315 20.9999 4.67157 19.8283C3.5 18.6568 3.5 16.7711 3.5 12.9999V8.49805C3.64488 8.50005 3.78999 8.49999 3.93114 8.49993L11.25 8.49992L11.25 15.0454L9.55748 13.1648C9.28038 12.8569 8.80617 12.832 8.49828 13.1091C8.1904 13.3862 8.16544 13.8604 8.44254 14.1683L11.4425 17.5016C11.5848 17.6596 11.7874 17.7499 12 17.7499C12.2126 17.7499 12.4152 17.6596 12.5575 17.5016L15.5575 14.1683C15.8346 13.8604 15.8096 13.3862 15.5017 13.1091C15.1938 12.832 14.7196 12.8569 14.4425 13.1648L12.75 15.0454L12.75 8.49992L20.0689 8.49993Z", stroke: "#7c3aed", fill: "#1C274C", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                        }
                    }
                    div { class: "fixed-info",
                        span { class: "fixed-title", "Playlists Locales" }
                        span { class: "fixed-count", "Importar listas de reproduccion" }
                    }
                }
                div {
                    class: "fixed-card",
                    onclick: move |_| async move {
                        if let Some(path) = commands::select_document().await {
                            if let Some(p) = commands::get_metadata(&path).await {
                                log(&format!("Metadata: title: {}\nArtista: {}\nDuracion: {}\nimagen: {}", p.title, p.artist, p.duration, p.image));
                                let track = TrackMetadata {
                                    title: p.title,
                                    artist: p.artist,
                                    duration_secs: p.duration,
                                    path: path.clone(),
                                    image: p.image,
                                };
                                let was_empty = player.read().user_queue.is_empty() && player.read().current_track().is_none();

                                player.write().add_to_queue(track);

                                if was_empty {
                                    let _ = commands::play_file(&path).await;
                                }
                            }
                        } else {
                            web_sys::console::log_1(&"select_document: cancelled".into());
                        }
                    },
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
                if playlists.is_empty() {
                    div { class: "empty-playlists-msg",
                        p { "No tienes listas guardadas aún." }
                    }
                } else {
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
                    img { src: "data:image/jpeg;base64,{url}", alt: "{playlist.title}" }
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
    let mut player = use_player_state();
    // let suggestions = suggested_songs();

    let play_all = {
        let tracks = playlist.tracks.clone();
        let title = playlist.title.clone();
        move |_| {
            if !tracks.is_empty() {
                let first_path = tracks[0].path.clone();
                player
                    .write()
                    .play_playlist(tracks.clone(), title.clone(), 0);
                if !first_path.is_empty() {
                    spawn(async move {
                        let _ = commands::play_file(&first_path).await;
                    });
                }
            }
        }
    };

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
                        img { src: "data:image/jpeg;base64,{url}", alt: "{playlist.title}" }
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
                    span { class: "detail-type", "Lista de reproducción" }
                    h1 { class: "detail-name", "{playlist.title}" }
                    span { class: "detail-author", "De {playlist.author}" }
                    span { class: "detail-stats", "{playlist.song_count} canciones / {playlist.duration}" }
                    div { class: "detail-actions",
                        button {
                            class: "detail-play-btn",
                            onclick: play_all,
                            svg { view_box: "0 0 24 24",
                                polygon { points: "5 3 19 12 5 21 5 3", stroke: "#ffffff", fill: "#ffffff", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                            }
                            "Reproducir"
                        }
                    }
                }
            }

    div { class: "track-list",
                div { class: "track-header",
                    span { "#" }
                    span { "Título" }
                    span { "Duración" }
                    span {}
                }
                for (idx, track) in playlist.tracks.iter().enumerate() {
                    {
                        let mins = (track.duration_secs / 60.0) as usize;
                        let secs = (track.duration_secs % 60.0) as usize;
                        let duration_fmt = format!("{mins}:{secs:02}");

                        let track_path = track.path.clone();
                        let all_tracks = playlist.tracks.clone();
                        let playlist_title = playlist.title.clone();

                        rsx! {
                            TrackRow {
                                key: "{idx}",
                                number: idx + 1,
                                title: track.title.clone(),
                                artist: track.artist.clone(),
                                duration: duration_fmt,
                                cover_url: if track.image.is_empty() { None } else { Some(track.image.clone()) },
                                on_play: move |_| {
                                    let path = track_path.clone();
                                    let tracks = all_tracks.clone();
                                    let title = playlist_title.clone();

                                    spawn(async move {
                                        player.write().play_playlist(tracks, title, idx);
                                        let _ = commands::play_file(&path).await;
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
}

#[component]
fn TrackRow(
    number: usize,
    title: String,
    artist: String,
    duration: String,
    cover_url: Option<String>,
    on_play: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "track-item",
            div {
                class: "track-number-wrap",
                onclick: move |_| on_play.call(()),
                span { class: "track-number", "{number}" }
                div { class: "track-play-icon",
                    svg { view_box: "0 0 24 24",
                        polygon { points: "5 3 19 12 5 21 5 3", stroke: "#ffffff", fill: "#ffffff", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                    }
                }
            }
            div { class: "track-info",
                div { class: "track-cover",
                    if let Some(ref url) = cover_url {
                        img { src: "data:image/jpeg;base64,{url}", alt: "{title}" }
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
