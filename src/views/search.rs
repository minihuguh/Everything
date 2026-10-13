use dioxus::prelude::*;
use crate::ipc::commands;
use crate::state::{use_player_state, TrackMetadata};
use crate::views::playlist::{get_cached_stat, Playlist};

#[component]
pub fn Search() -> Element {
    let mut query = use_signal(String::new);
    let active_filter = use_signal(|| "todo");
    let mut player = use_player_state();

    let playlists_resource = use_resource(|| async move {
        let p = get_cached_stat().await;
        if p.is_empty() {
            return Vec::new();
        }

        let mut mapped_playlists = Vec::new();

        for sp in p {
            let mut loaded_tracks = Vec::new();

            // The backend cache is leveraged to implement the images.
            for st in sp.tracks {
                let mut img = st.image;
                if img.is_empty() && !st.path.is_empty() {
                    if let Some(meta) = commands::get_metadata(&st.path).await {
                        if !meta.image.is_empty() {
                            img = meta.image;
                        }
                    }
                }

                loaded_tracks.push(TrackMetadata {
                    title: st.title,
                    artist: st.artist,
                    duration_secs: st.duration_secs,
                    path: st.path,
                    image: img,
                });
            }

            let total_secs = loaded_tracks.iter().map(|t| t.duration_secs).sum::<f64>();
            let mins = (total_secs / 60.0) as usize;

            mapped_playlists.push(Playlist {
                id: sp.id,
                title: sp.title,
                author: sp.author,
                song_count: loaded_tracks.len(),
                duration: format!("{mins}m"),
                cover_url: loaded_tracks
                    .first()
                    .map(|t| t.image.clone())
                    .filter(|img| !img.is_empty()),
                path: sp.file_path,
                is_downloaded: true,
                tracks: loaded_tracks,
            });
        }

        mapped_playlists
    });

    let playlists = playlists_resource.value().read().clone().unwrap_or_default();

    let search_query = query().to_lowercase();
    let is_searching = !search_query.is_empty();
    let filter = *active_filter.read();

    // Filtrado reactivo de playlists y canciones dentro de las playlists
    let mut matched_playlists = Vec::new();
    let mut matched_songs = Vec::new();

    if is_searching {
        for pl in &playlists {
            if pl.title.to_lowercase().contains(&search_query) {
                matched_playlists.push(pl.clone());
            }

            for track in &pl.tracks {
                if track.title.to_lowercase().contains(&search_query) ||
                    track.artist.to_lowercase().contains(&search_query) {
                    matched_songs.push((pl.title.clone(), track.clone(), pl.tracks.clone()));
                }
            }
        }
    }

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/search.css") }

        div { class: "search-view",
            // Header con título
            h1 { class: "search-heading", "Buscar" }

            // Barra de búsqueda estilo Spotube
            div { class: "search-bar",
                svg { class: "search-icon", width: "20", height: "20", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
                    circle { cx: "11", cy: "11", r: "8" }
                    line { x1: "21", y1: "21", x2: "16.65", y2: "16.65" }
                }
                input {
                    class: "search-input",
                    placeholder: "¿Qué quieres escuchar?",
                    value: "{query()}",
                    oninput: move |e| query.set(e.value().clone())
                }
                if !query().is_empty() {
                    button {
                        class: "search-clear",
                        onclick: move |_| query.set(String::new()),
                        "✕"
                    }
                }
            }

            // Filtros
            div { class: "search-filters",
                FilterButton { label: "Todo", filter: "todo", active_filter }
                FilterButton { label: "Canciones", filter: "songs", active_filter }
                FilterButton { label: "Álbumes", filter: "albums", active_filter }
                FilterButton { label: "Artistas", filter: "artists", active_filter }
                FilterButton { label: "Playlists", filter: "playlists", active_filter }
            }

            if is_searching {
                if (filter == "todo" || filter == "playlists") && !matched_playlists.is_empty() {
                    div { class: "search-section",
                        h2 { class: "section-title", "Playlists encontradas" }
                        div { class: "result-grid",
                            for pl in matched_playlists.iter() {
                                ResultCard {
                                    title: &*pl.title.clone(),
                                    artist: &*pl.author.clone(),
                                    image_url: pl.cover_url.clone(),
                                    type_: "Playlist",
                                    on_click: move |_| {
                                        // From here, the ability to play playlists should be added (to be implemented).
                                    }
                                }
                            }
                        }
                    }
                }

                if (filter == "todo" || filter == "songs") && !matched_songs.is_empty() {
                    div { class: "search-section",
                        h2 { class: "section-title", "Canciones en tus playlists" }
                        div { class: "result-grid",
                            for (pl_title, track, all_tracks) in matched_songs.iter() {
                                {
                                    let track_path = track.path.clone();
                                    let t_clone = track.clone();
                                    let p_title = pl_title.clone();
                                    let tracks_list = all_tracks.clone();

                                    rsx! {
                                        ResultCard {
                                            title: t_clone.title.clone(),
                                            artist: format!("{} • En: {}", t_clone.artist, p_title),
                                            image_url: if t_clone.image.is_empty() { None } else { Some(t_clone.image) },
                                            type_: "Canción",
                                            on_click: move |_| {
                                                let path = track_path.clone();
                                                let t_list = tracks_list.clone();
                                                let playlist_name = p_title.clone();

                                                // We locate the song's index within your playlist to play it correctly
                                                let idx = t_list.iter().position(|t| t.path == path).unwrap_or(0);

                                                spawn(async move {
                                                    player.write().play_playlist(t_list, playlist_name, idx);
                                                    if !path.is_empty() {
                                                        let _ = commands::play_file(&path).await;
                                                    }
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if matched_playlists.is_empty() && matched_songs.is_empty() {
                    div { class: "empty-playlists-msg", style: "text-align: center; padding: 40px; color: #808090;",
                        p { "No se encontraron resultados para \"{query()}\"" }
                    }
                }
            } else { div { class: "search-section",
                h2 { class: "section-title", "Explorar por género" }

                div { class: "genre-grid",
                    GenreCard { name: "Pop", color: "#e91e63" }
                    GenreCard { name: "Rock", color: "#f44336" }
                    GenreCard { name: "Hip Hop", color: "#ff9800" }
                    GenreCard { name: "Electrónica", color: "#9c27b0" }
                    GenreCard { name: "Jazz", color: "#3f51b5" }
                    GenreCard { name: "Clásica", color: "#009688" }
                    GenreCard { name: "Lo-Fi", color: "#795548" }
                    GenreCard { name: "Reggaetón", color: "#ff5722" }
                }
            }
        }
    }
}
}

#[component]
fn FilterButton(label: &'static str, filter: &'static str, active_filter: Signal<&'static str>) -> Element {
    let is_active = active_filter() == filter;

    rsx! {
        button {
            class: if is_active { "filter-btn active" } else { "filter-btn" },
            onclick: move |_| active_filter.set(filter),
            "{label}"
        }
    }
}

#[component]
fn ResultCard(
    title: String,
    artist: String,
    image_url: Option<String>,
    type_: &'static str,
    on_click: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: "result-card",
            onclick: move |_| on_click.call(()),
            div { class: "result-cover",
                if let Some(ref url) = image_url {
                    img { src: "data:image/jpeg;base64,{url}", alt: "{title}", style: "width: 100%; height: 100%; object-fit: cover;" }
                } else {
                    div { class: "result-image",
                        svg { width: "24", height: "24", view_box: "0 0 24 24", fill: "none", stroke: "#7c3aed", stroke_width: "2",
                            path { d: "M9 18V5l12-2v13" }
                            circle { cx: "6", cy: "18", r: "3" }
                            circle { cx: "18", cy: "16", r: "3" }
                        }
                    }
                }
                div { class: "result-play",
                    svg { width: "24", height: "24", view_box: "0 0 24 24", fill: "currentColor",
                        path { d: "M8 5v14l11-7z" }
                    }
                }
            }
            div { class: "result-info",
                div { class: "result-title", "{title}" }
                div { class: "result-meta",
                    span { class: "result-type", "{type_}" }
                    span { class: "result-dot", "•" }
                    span { class: "result-artist", "{artist}" }
                }
            }
        }
    }
}

#[component]
fn GenreCard(name: &'static str, color: &'static str) -> Element {
    rsx! {
        div { 
            class: "genre-card",
            style: "background: linear-gradient(135deg, {color}, {color}88)",
            "{name}"
        }
    }
}