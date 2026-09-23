use crate::state::use_player_state;
use parser::Playlist;

use chrono::Timelike;
use dioxus::prelude::*;
use crate::ipc::commands;
// ============================================================
// Tipos de datos
// ============================================================

#[derive(Clone, PartialEq)]
pub struct QuickAccessItem {
    pub title: &'static str,
    pub icon: &'static str,
    pub cover_url: Option<&'static str>,
    pub on_play: Option<EventHandler<MouseEvent>>
}

#[derive(Clone, PartialEq)]
pub struct CarouselItemData {
    pub title: &'static str,
    pub subtitle: &'static str,
    pub cover_url: Option<&'static str>,
    pub icon: &'static str,
}

// ============================================================
// Datos de ejemplo
// ============================================================

pub fn quick_access_items() -> Vec<QuickAccessItem> {
    vec![
        QuickAccessItem {
            title: "Tus Me Gusta",
            icon: "♥",
            cover_url: None,
            on_play: None,
        },
        QuickAccessItem {
            title: "Lo más escuchado",
            icon: "📈",
            cover_url: None,
            on_play: None,
        },
        QuickAccessItem {
            title: "Historial",
            icon: "🕐",
            cover_url: None,
            on_play: None,
        },
        QuickAccessItem {
            title: "Descubrimiento",
            icon: "✨",
            cover_url: None,
            on_play: None,
        },
        QuickAccessItem {
            title: "Entrenamiento",
            icon: "💪",
            cover_url: None,
            on_play: None,
        },
    ]
}

pub fn recent_items() -> Vec<CarouselItemData> {
    vec![
        CarouselItemData {
            title: "After Hours",
            subtitle: "The Weeknd",
            cover_url: None,
            icon: "🎵",
        },
        CarouselItemData {
            title: "Random Access Memories",
            subtitle: "Daft Punk",
            cover_url: None,
            icon: "🎹",
        },
        CarouselItemData {
            title: "Currents",
            subtitle: "Tame Impala",
            cover_url: None,
            icon: "🎸",
        },
        CarouselItemData {
            title: "Rumours",
            subtitle: "Fleetwood Mac",
            cover_url: None,
            icon: "🥁",
        },
        CarouselItemData {
            title: "Blue Train",
            subtitle: "John Coltrane",
            cover_url: None,
            icon: "🎺",
        },
        CarouselItemData {
            title: "Kind of Blue",
            subtitle: "Miles Davis",
            cover_url: None,
            icon: "🎷",
        },
    ]
}

pub fn continue_items() -> Vec<CarouselItemData> {
    vec![
        CarouselItemData {
            title: "Mix Diario 1",
            subtitle: "Hecho para ti",
            cover_url: None,
            icon: "🔀",
        },
        CarouselItemData {
            title: "Descubre Indie",
            subtitle: "Playlist",
            cover_url: None,
            icon: "🎧",
        },
        CarouselItemData {
            title: "Lo-Fi Beats",
            subtitle: "Para estudiar",
            cover_url: None,
            icon: "📚",
        },
        CarouselItemData {
            title: "Rock Clásico",
            subtitle: "Playlist",
            cover_url: None,
            icon: "🎸",
        },
        CarouselItemData {
            title: "Electrónica 2026",
            subtitle: "Actualizado",
            cover_url: None,
            icon: "⚡",
        },
        CarouselItemData {
            title: "Jazz Lounge",
            subtitle: "Relajación",
            cover_url: None,
            icon: "🎺",
        },
    ]
}

// ============================================================
// Helpers
// ============================================================

fn greeting_text() -> String {
    let hour = chrono::Local::now().hour();
    match hour {
        6..=11 => "Buenos días".to_string(),
        12..=17 => "Buenas tardes".to_string(),
        _ => "Buenas noches".to_string(),
    }
}

// ============================================================
// Componentes
// ============================================================

#[component]
pub fn Home() -> Element {
    let mut quick = quick_access_items();
    let recent = recent_items();
    let cont = continue_items();

    let path = use_signal(|| {
        String::from(
            "C:\\Users\\PC\\Downloads\\J V N - You Got Me (Official Audio) [D1_686Kpj9w].opus",
        )
    });

    let player = use_player_state();

    let on_play = move |_: MouseEvent| commands::playlist();

    // quick[2] = QuickAccessItem {
    //     title: "Historial",
    //     icon: "🕐",
    //     cover_url: None,
    //     on_play: Some(EventHandler::new(on_play)),
    // };

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/home.css") }

        div { class: "home-container",
            GreetingHeader {}
            QuickAccessGrid { items: quick }
            HorizontalCarousel {
                title: "Escuchado recientemente",
                items: recent,
            }
            HorizontalCarousel {
                title: "Continuar escuchando",
                items: cont,
            }
        }
    }
}

#[component]
fn GreetingHeader() -> Element {
    let greeting = use_memo(greeting_text);

    rsx! {
        div {
            h1 { class: "greeting-header", "{greeting}" }
            p { class: "greeting-sub", "Aquí tienes lo que necesitas para empezar" }
        }
    }
}

#[component]
fn QuickAccessGrid(items: Vec<QuickAccessItem>) -> Element {
    rsx! {
        div { class: "quick-access-grid",
            for (idx, item) in items.iter().enumerate() {
                QuickAccessCard {
                    key: "{idx}",
                    title: item.title,
                    icon: item.icon,
                    cover_url: item.cover_url,
                    on_play: item.on_play.clone(),
                }
            }
        }
    }
}

#[component]
fn QuickAccessCard(
    title: &'static str,
    icon: &'static str,
    cover_url: Option<&'static str>,
    on_play: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! {
        div {
            class: "quick-access-card",
            onclick: move |evt| {
                if let Some(handler) = on_play {
                    handler.call(evt);
                }
            },
            div { class: "quick-access-cover",
                if let Some(url) = cover_url {
                    img { src: "{url}", alt: "{title}" }
                } else {
                    div { class: "cover-placeholder", "{icon}" }
                }
            }
            span { class: "quick-access-text", "{title}" }
        }
    }
}

#[component]
fn HorizontalCarousel(
    title: &'static str,
    items: Vec<CarouselItemData>,
) -> Element {
    rsx! {
        div { class: "section-carousel",
            div { class: "carousel-header",
                h2 { class: "carousel-title", "{title}" }
                button { class: "carousel-see-all", "Ver todo" }
            }
            div { class: "carousel-row",
                for (idx, item) in items.iter().enumerate() {
                    CarouselItem {
                        key: "{idx}",
                        title: item.title,
                        subtitle: item.subtitle,
                        cover_url: item.cover_url,
                        icon: item.icon,
                    }
                }
            }
        }
    }
}

#[component]
fn CarouselItem(
    title: &'static str,
    subtitle: &'static str,
    cover_url: Option<&'static str>,
    icon: &'static str,
) -> Element {
    rsx! {
        div {
            class: "carousel-item",
            onclick: move |_| {
                // TODO: reproducir o abrir detalle
            },
            div { class: "carousel-cover",
                if let Some(url) = cover_url {
                    img { src: "{url}", alt: "{title}" }
                } else {
                    div { class: "cover-placeholder", "{icon}" }
                }
            }
            div { class: "carousel-info",
                span { class: "carousel-item-title", "{title}" }
                span { class: "carousel-item-subtitle", "{subtitle}" }
            }
        }
    }
}

