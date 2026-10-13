use std::sync::Mutex;
use dioxus::prelude::*;
use shared::{StatisticsData, StatMetric, TopSong, TopArtist, ComparisonBar};
use crate::ipc::commands::get_stat;

static STATS_CACHE: Mutex<Option<StatisticsData>> = Mutex::new(None);

pub async fn get_cached_stat() -> StatisticsData {
    {
        let cache = STATS_CACHE.lock().unwrap();
        if let Some(ref data) = *cache {
            return data.clone();
        }
    }

    if let Some(data) = get_stat().await {
        let mut cache = STATS_CACHE.lock().unwrap();
        *cache = Some(data.clone());
        return data;
    }
    
    StatisticsData {
        filters: vec![],
        data_by_filter: std::collections::HashMap::new(),
    }
}

#[component]
pub fn Stadistics() -> Element {
    let stats_data = use_resource(move || async move {
        get_cached_stat().await
    });

    let mut active_filter = use_signal(|| "7d".to_string());
    let current_filter_key = active_filter();

    let stats = stats_data
        .value()
        .read()
        .as_ref()
        .cloned()
        .unwrap_or_else(|| StatisticsData {
            filters: vec![],
            data_by_filter: std::collections::HashMap::new(),
        });

    let current_metrics = stats
        .data_by_filter
        .get(&current_filter_key)
        .cloned()
        .unwrap_or_default();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/stadistics.css") }

        div { class: "stats-container",
            TimeFilters {
                filters: stats.filters,
                active_filter
            }

            SummaryGrid { metrics: current_metrics.metrics }

            div { class: "tops-grid",
                TopSongsList { songs: current_metrics.top_songs }
                TopArtistsList { artists: current_metrics.top_artists }
            }

            MonthlyComparison { comparison: current_metrics.comparison }
            GeneratePlaylistButton {}
        }
    }
}

#[component]
fn TimeFilters(filters: Vec<shared::TimeFilter>, active_filter: Signal<String>) -> Element {
    rsx! {
        div { class: "time-filters",
            for filter in filters.iter() {
                FilterButton {
                    key: "{filter.value}",
                    value: filter.value.clone(),
                    label: filter.label.clone(),
                    active_filter: active_filter,
                }
            }
        }
    }
}

#[component]
fn FilterButton(value: String, label: String, mut active_filter: Signal<String>) -> Element {
    let is_active = value == *active_filter.read();
    rsx! {
        button {
            class: if is_active { "time-filter-btn active" } else { "time-filter-btn" },
            onclick: move |_| active_filter.set(value.clone()),
            "{label}"
        }
    }
}

#[component]
fn SummaryGrid(metrics: Vec<StatMetric>) -> Element {
    rsx! {
        div { class: "stats-summary-grid",
            for (idx, metric) in metrics.iter().enumerate() {
                StatCard {
                    key: "{idx}",
                    label: &*metric.label.clone(),
                    value: &*metric.value.clone(),
                    sub: &*metric.sub.clone(),
                }
            }
        }
    }
}

#[component]
fn StatCard(label: String, value: String, sub: String) -> Element {
    // Iconos dinámicos basados en la etiqueta de la métrica
    let icon_svg = match label.as_str() {
        "Tiempo total" => "<svg viewBox='0 0 24 24'><circle cx='12' cy='12' r='10'/><polyline points='12 6 12 12 16 14'/></svg>",
        "Canciones y artistas" => "<svg viewBox='0 0 24 24'><path d='M9 18V5l12-2v13'/><circle cx='6' cy='18' r='3'/><circle cx='18' cy='16' r='3'/></svg>",
        _ => "<svg viewBox='0 0 24 24'><path d='M13 2L3 14h9l-1 8 10-12h-9l1-8z'/></svg>",
    };

    rsx! {
        div { class: "stat-card",
            div { class: "stat-card-header",
                div { class: "stat-icon",
                    dangerous_inner_html: "{icon_svg}"
                }
                span { class: "stat-label", "{label}" }
            }
            span { class: "stat-value", "{value}" }
            span { class: "stat-sub", "{sub}" }
        }
    }
}

#[component]
fn TopSongsList(songs: Vec<TopSong>) -> Element {
    rsx! {
        div { class: "stats-section",
            div { class: "stats-section-header",
                h2 { class: "stats-section-title", "Top Canciones" }
                span { class: "stats-section-subtitle", "Más reproducidas" }
            }
            div { class: "top-list",
                for song in songs.iter() {
                    TopSongItem {
                        key: "{song.rank}",
                        rank: song.rank,
                        title: &*song.title.clone(),
                        artist: &*song.artist.clone(),
                        plays: &*song.plays.clone(),
                        cover_url: song.cover_url.clone(),
                    }
                }
            }
        }
    }
}

#[component]
fn TopSongItem(rank: usize, title: String, artist: String, plays: String, cover_url: Option<String>) -> Element {
    let rank_class = match rank {
        1 => "top-rank gold",
        2 => "top-rank silver",
        3 => "top-rank bronze",
        _ => "top-rank",
    };

    rsx! {
        div { class: "top-item",
            div { class: "{rank_class}", "{rank}" }
            div { class: "top-cover",
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
            div { class: "top-info",
                span { class: "top-title", "{title}" }
                span { class: "top-subtitle", "{artist}" }
            }
            span { class: "top-meta", "{plays} reprod." }
        }
    }
}

#[component]
fn TopArtistsList(artists: Vec<TopArtist>) -> Element {
    rsx! {
        div { class: "stats-section",
            div { class: "stats-section-header",
                h2 { class: "stats-section-title", "Top Artistas" }
                span { class: "stats-section-subtitle", "Por tiempo de escucha" }
            }
            div { class: "top-list",
                for artist in artists.iter() {
                    TopArtistItem {
                        key: "{artist.rank}",
                        rank: artist.rank,
                        name: artist.name.clone(),
                        time: artist.time.clone(),
                        cover_url: artist.cover_url.clone(),
                    }
                }
            }
        }
    }
}

#[component]
fn TopArtistItem(rank: usize, name: String, time: String, cover_url: Option<String>) -> Element {
    let rank_class = match rank {
        1 => "top-rank gold",
        2 => "top-rank silver",
        3 => "top-rank bronze",
        _ => "top-rank",
    };

    rsx! {
        div { class: "top-item",
            div { class: "{rank_class}", "{rank}" }
            div { class: "top-cover",
                if let Some(url) = cover_url {
                    img { src: "{url}", alt: "{name}" }
                } else {
                    svg { view_box: "0 0 24 24",
                        path { d: "M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2", stroke: "#7c3aed", fill: "none", "stroke-width": "2", "stroke-linecap": "round" }
                        circle { cx: "12", cy: "7", r: "4", stroke: "#7c3aed", fill: "none", "stroke-width": "2" }
                    }
                }
            }
            div { class: "top-info",
                span { class: "top-title", "{name}" }
            }
            span { class: "top-meta", "{time}" }
        }
    }
}

#[component]
fn MonthlyComparison(comparison: Vec<ComparisonBar>) -> Element {
    let max_val = comparison.iter().map(|c| c.value).fold(0.0, f64::max);

    rsx! {
        div { class: "stats-section",
            div { class: "stats-section-header",
                h2 { class: "stats-section-title", "Comparativa" }
            }
            div { class: "comparison-container",
                div { class: "comparison-bars",
                    for bar in comparison.iter() {
                        ComparisonRow {
                            key: "{bar.label}",
                            label: bar.label.clone(),
                            value: bar.value,
                            display: bar.display.clone(),
                            max_val,
                            is_current: bar.is_current,
                            delta: bar.delta.clone(),
                            delta_type: bar.delta_type.clone(),
                        }
                    }
                }
            }
        }
    }
}
#[component]
fn ComparisonRow(label: String, value: f64, display: String, max_val: f64, is_current: bool, delta: String, delta_type: String) -> Element {
    let pct = if max_val > 0.0 { (value / max_val) * 100.0 } else { 0.0 };
    let fill_class = if is_current { "comparison-fill" } else { "comparison-fill secondary" };
    let delta_class = match delta_type.as_str() {
        "up" => "comparison-delta up",
        "down" => "comparison-delta down",
        _ => "comparison-delta neutral",
    };

    rsx! {
        div { class: "comparison-row",
            span { class: "comparison-label", "{label}" }
            div { class: "comparison-track",
                div { class: "{fill_class}", style: "width: {pct}%",
                    span { "{display}" }
                }
            }
            if !delta.is_empty() {
                span { class: "{delta_class}", "{delta}" }
            }
        }
    }
}

#[component]
fn GeneratePlaylistButton() -> Element {
    rsx! {
        div { class: "stats-section",
            button {
                class: "generate-btn",
                onclick: move |_| {
                    // TODO: generar playlist desde top 50
                },
                svg { view_box: "0 0 24 24",
                    path { d: "M12 5v14M5 12h14", stroke: "#ffffff", fill: "none", "stroke-width": "2", "stroke-linecap": "round" }
                }
                "Crear playlist del top 50"
            }
        }
    }
}