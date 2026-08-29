use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct TimeFilter {
    pub label: &'static str,
    pub value: &'static str,
}

#[derive(Clone, PartialEq)]
pub struct StatMetric {
    pub label: &'static str,
    pub value: &'static str,
    pub sub: &'static str,
    pub icon_svg: &'static str,
}

#[derive(Clone, PartialEq)]
pub struct TopSong {
    pub rank: usize,
    pub title: &'static str,
    pub artist: &'static str,
    pub plays: &'static str,
    pub cover_url: Option<&'static str>,
}

#[derive(Clone, PartialEq)]
pub struct TopArtist {
    pub rank: usize,
    pub name: &'static str,
    pub time: &'static str,
    pub cover_url: Option<&'static str>,
}

#[derive(Clone, PartialEq)]
pub struct HeatmapCell {
    pub level: u8,
}

#[derive(Clone, PartialEq)]
pub struct ComparisonBar {
    pub label: &'static str,
    pub value: f64,
    pub display: &'static str,
    pub is_current: bool,
    pub delta: &'static str,
    pub delta_type: &'static str,
}

pub fn time_filters() -> Vec<TimeFilter> {
    vec![
        TimeFilter { label: "Últimos 7 días", value: "7d" },
        TimeFilter { label: "Últimos 30 días", value: "30d" },
        TimeFilter { label: "Este año", value: "year" },
        TimeFilter { label: "Histórico", value: "all" },
    ]
}

pub fn summary_metrics() -> Vec<StatMetric> {
    vec![
        StatMetric {
            label: "Tiempo total",
            value: "342h 18m",
            sub: "+12% vs mes anterior",
            icon_svg: "<svg viewBox='0 0 24 24'><circle cx='12' cy='12' r='10'/><polyline points='12 6 12 12 16 14'/></svg>",
        },
        StatMetric {
            label: "Canciones y artistas",
            value: "1,247 / 312",
            sub: "Canciones reproducidas / Artistas distintos",
            icon_svg: "<svg viewBox='0 0 24 24'><path d='M9 18V5l12-2v13'/><circle cx='6' cy='18' r='3'/><circle cx='18' cy='16' r='3'/></svg>",
        },
        StatMetric {
            label: "Racha actual",
            value: "23 días",
            sub: "Récord: 45 días",
            icon_svg: "<svg viewBox='0 0 24 24'><path d='M13 2L3 14h9l-1 8 10-12h-9l1-8z'/></svg>",
        },
    ]
}

pub fn top_songs() -> Vec<TopSong> {
    vec![
        TopSong { rank: 1, title: "Blinding Lights", artist: "The Weeknd", plays: "142", cover_url: None },
        TopSong { rank: 2, title: "Levitating", artist: "Dua Lipa", plays: "118", cover_url: None },
        TopSong { rank: 3, title: "Stay", artist: "The Kid LAROI", plays: "97", cover_url: None },
        TopSong { rank: 4, title: "Peaches", artist: "Justin Bieber", plays: "84", cover_url: None },
        TopSong { rank: 5, title: "Good 4 U", artist: "Olivia Rodrigo", plays: "76", cover_url: None },
    ]
}

pub fn top_artists() -> Vec<TopArtist> {
    vec![
        TopArtist { rank: 1, name: "The Weeknd", time: "48h 12m", cover_url: None },
        TopArtist { rank: 2, name: "Daft Punk", time: "36h 45m", cover_url: None },
        TopArtist { rank: 3, name: "Tame Impala", time: "29h 30m", cover_url: None },
        TopArtist { rank: 4, name: "Dua Lipa", time: "22h 15m", cover_url: None },
        TopArtist { rank: 5, name: "Fleetwood Mac", time: "18h 40m", cover_url: None },
    ]
}

pub fn heatmap_data() -> Vec<Vec<HeatmapCell>> {
    vec![
        vec![HeatmapCell { level: 0 }, HeatmapCell { level: 1 }, HeatmapCell { level: 2 }, HeatmapCell { level: 3 }, HeatmapCell { level: 1 }, HeatmapCell { level: 0 }, HeatmapCell { level: 2 }],
        vec![HeatmapCell { level: 1 }, HeatmapCell { level: 3 }, HeatmapCell { level: 4 }, HeatmapCell { level: 2 }, HeatmapCell { level: 3 }, HeatmapCell { level: 1 }, HeatmapCell { level: 0 }],
        vec![HeatmapCell { level: 2 }, HeatmapCell { level: 4 }, HeatmapCell { level: 5 }, HeatmapCell { level: 4 }, HeatmapCell { level: 2 }, HeatmapCell { level: 1 }, HeatmapCell { level: 1 }],
        vec![HeatmapCell { level: 1 }, HeatmapCell { level: 2 }, HeatmapCell { level: 3 }, HeatmapCell { level: 5 }, HeatmapCell { level: 4 }, HeatmapCell { level: 2 }, HeatmapCell { level: 0 }],
        vec![HeatmapCell { level: 0 }, HeatmapCell { level: 1 }, HeatmapCell { level: 2 }, HeatmapCell { level: 3 }, HeatmapCell { level: 2 }, HeatmapCell { level: 1 }, HeatmapCell { level: 0 }],
    ]
}

pub fn comparison_data() -> Vec<ComparisonBar> {
    vec![
        ComparisonBar { label: "Este mes", value: 78.5, display: "78h 30m", is_current: true, delta: "+15%", delta_type: "up" },
        ComparisonBar { label: "Mes pasado", value: 68.2, display: "68h 12m", is_current: false, delta: "", delta_type: "neutral" },
    ]
}

#[component]
pub fn Stadistics() -> Element {
    let filters = time_filters();
    let metrics = summary_metrics();
    let songs = top_songs();
    let artists = top_artists();
    let heatmap = heatmap_data();
    let comparison = comparison_data();
    let mut active_filter = use_signal(|| "30d");

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/stadistics.css") }

        div { class: "stats-container",
            TimeFilters { filters, active_filter }
            SummaryGrid { metrics }

            div { class: "tops-grid",
                TopSongsList { songs }
                TopArtistsList { artists }
            }

            // ActivityHeatmap { heatmap }
            // MoodIndicator {}
            MonthlyComparison { comparison }
            GeneratePlaylistButton {}
        }
    }
}

#[component]
fn TimeFilters(filters: Vec<TimeFilter>, active_filter: Signal<&'static str>) -> Element {
    rsx! {
        div { class: "time-filters",
            for (idx, filter) in filters.iter().enumerate() {
                FilterButton {
                    key: "{idx}",
                    value: filter.value,
                    label: filter.label,
                    active_filter,
                }
            }
        }
    }
}

#[component]
fn FilterButton(value: &'static str, label: &'static str, active_filter: Signal<&'static str>) -> Element {
    let is_active = value == active_filter();
    rsx! {
        button {
            class: if is_active { "time-filter-btn active" } else { "time-filter-btn" },
            onclick: move |_| active_filter.set(value),
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
                    label: metric.label,
                    value: metric.value,
                    sub: metric.sub,
                    icon_svg: metric.icon_svg,
                }
            }
        }
    }
}

#[component]
fn StatCard(label: &'static str, value: &'static str, sub: &'static str, icon_svg: &'static str) -> Element {
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
                        rank: song.rank,
                        title: song.title,
                        artist: song.artist,
                        plays: song.plays,
                        cover_url: song.cover_url,
                    }
                }
            }
        }
    }
}

#[component]
fn TopSongItem(rank: usize, title: &'static str, artist: &'static str, plays: &'static str, cover_url: Option<&'static str>) -> Element {
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
                        rank: artist.rank,
                        name: artist.name,
                        time: artist.time,
                        cover_url: artist.cover_url,
                    }
                }
            }
        }
    }
}

#[component]
fn TopArtistItem(rank: usize, name: &'static str, time: &'static str, cover_url: Option<&'static str>) -> Element {
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
fn ActivityHeatmap(heatmap: Vec<Vec<HeatmapCell>>) -> Element {
    let days = vec!["Lun", "Mar", "Mié", "Jue", "Vie", "Sáb", "Dom"];

    rsx! {
        div { class: "stats-section",
            div { class: "stats-section-header",
                h2 { class: "stats-section-title", "Patrón de actividad" }
                span { class: "stats-section-subtitle", "Últimas 5 semanas" }
            }
            div { class: "heatmap-container",
                div { class: "heatmap-grid",
                    for row in heatmap.iter() {
                        for cell in row.iter() {
                            div { class: "heatmap-cell level-{cell.level}" }
                        }
                    }
                }
                div { class: "heatmap-labels",
                    for day in days.iter() {
                        span { "{day}" }
                    }
                }
            }
        }
    }
}

#[component]
fn MoodIndicator() -> Element {
    let mood_value = 72.0;
    let mood_label = if mood_value > 60.0 { "Descubridor" } else { "Zona de confort" };

    rsx! {
        div { class: "stats-section",
            div { class: "stats-section-header",
                h2 { class: "stats-section-title", "Estilo de escucha" }
            }
            div { class: "mood-container",
                div { class: "mood-current", "{mood_label}" }
                div { class: "mood-bar-track",
                    div { class: "mood-bar-fill", style: "width: {mood_value}%" }
                }
                div { class: "mood-labels",
                    span { "Zona de confort" }
                    span { "Descubridor" }
                }
            }
        }
    }
}

#[component]
fn MonthlyComparison(comparison: Vec<ComparisonBar>) -> Element {
    let max_val = comparison.iter().map(|c| c.value).fold(0.0, f64::max);

    rsx! {
        div { class: "stats-section",
            div { class: "stats-section-header",
                h2 { class: "stats-section-title", "Comparativa mensual" }
            }
            div { class: "comparison-container",
                div { class: "comparison-bars",
                    for bar in comparison.iter() {
                        ComparisonRow {
                            label: bar.label,
                            value: bar.value,
                            display: bar.display,
                            max_val,
                            is_current: bar.is_current,
                            delta: bar.delta,
                            delta_type: bar.delta_type,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ComparisonRow(label: &'static str, value: f64, display: &'static str, max_val: f64, is_current: bool, delta: &'static str, delta_type: &'static str) -> Element {
    let pct = if max_val > 0.0 { (value / max_val) * 100.0 } else { 0.0 };
    let fill_class = if is_current { "comparison-fill" } else { "comparison-fill secondary" };
    let delta_class = match delta_type {
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
