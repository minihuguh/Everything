use dioxus::prelude::*;

#[component]
pub fn Search() -> Element {
    let mut query = use_signal(|| String::new());
    let mut active_filter = use_signal(|| "todo");

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
                    oninput: move |e| query.set(e.value().to_string())
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

            // Resultados recientes
            div { class: "search-section",
                div { class: "section-header",
                    h2 { class: "section-title", "Resultados recientes" }
                    button { class: "section-clear", "Borrar todo" }
                }

                div { class: "result-grid",
                    ResultCard { 
                        title: "Midnight City", 
                        artist: "M83", 
                        image: "🌃",
                        type_: "Canción"
                    }
                    ResultCard { 
                        title: "After Hours", 
                        artist: "The Weeknd", 
                        image: "🌆",
                        type_: "Álbum"
                    }
                    ResultCard { 
                        title: "Lo-Fi Beats", 
                        artist: "Spotube", 
                        image: "🎧",
                        type_: "Playlist"
                    }
                    ResultCard { 
                        title: "Daft Punk", 
                        artist: "Artista", 
                        image: "🤖",
                        type_: "Artista"
                    }
                    ResultCard { 
                        title: "Random Access Memories", 
                        artist: "Daft Punk", 
                        image: "💿",
                        type_: "Álbum"
                    }
                    ResultCard { 
                        title: "Nightcall", 
                        artist: "Kavinsky", 
                        image: "🌙",
                        type_: "Canción"
                    }
                }
            }

            // Géneros populares
            div { class: "search-section",
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
fn ResultCard(title: &'static str, artist: &'static str, image: &'static str, type_: &'static str) -> Element {
    rsx! {
        div { class: "result-card",
            div { class: "result-cover",
                div { class: "result-image", "{image}" }
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