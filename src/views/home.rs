// use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::window;
use serde::Deserialize;
use serde_json::Value;
use crate::state::{use_player_state, TrackMetadata};
//
// ─── Acceso seguro a Tauri via web_sys ───
fn get_tauri_invoke() -> Option<js_sys::Function> {
    let win = window()?;
    let tauri = js_sys::Reflect::get(&win, &"__TAURI__".into()).ok()?;
    if tauri.is_undefined() || tauri.is_null() {
        return None;
    }
    let core = js_sys::Reflect::get(&tauri, &"core".into()).ok();
    let invoke_target = core.as_ref().unwrap_or(&tauri);
    let invoke = js_sys::Reflect::get(invoke_target, &"invoke".into()).ok()?;
    if invoke.is_function() { Some(invoke.into()) } else { None }
}

async fn tauri_invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue> {
    // let invoke_fn = if let Some(f) = get_tauri_invoke() { f } else {
    //     web_sys::console::log_1(&format!("[TAURI NOT AVAILABLE] {cmd}").into());
    //     return Ok(JsValue::NULL);
    // };

    let Some(invoke_fn) = get_tauri_invoke() else {
        web_sys::console::log_1(&format!("[TAURI NOT AVAILABLE] {cmd}").into());
        return Ok(JsValue::NULL);
    };
    let cmd_js = JsValue::from_str(cmd);
    let this = JsValue::NULL;
    let promise = invoke_fn.call2(&this, &cmd_js, &args)?;
    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise)).await
}
//
// ─── Structs tipados para parseo robusto ───
#[derive(Debug, Deserialize)]
struct PlayResponse {
    metadata: Option<Metadata>,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    title: String,
    artist: String,
    #[serde(rename = "duration")]
    duration_secs: f64,
    image: String,
}

//
// #[component]
// pub fn Home() -> Element {
//     let mut path = use_signal(|| {
//         String::from(
//             "C:\\Users\\PC\\Downloads\\Alesso - Take My Breath Away (Lyric Video) [o10EV4PG40U].webm",
//         )
//     });
//
//     let player = use_player_state();
//
//     fn log(msg: &str) {
//         web_sys::console::log_1(&format!("[HOME] {msg}").into());
//     }
//
//     // ─── Handler reproducir ───
//     let on_play = move |_| {
//         let path_val = path();
//         let mut player = player;
//
//         async move {
//             log("=== PLAY iniciado ===");
//             player.with_mut(|s| s.is_loading = true);
//
//             // Serializar args con serde_json (escape automático del path)
//             let args = match serde_wasm_bindgen::to_value(
//                 &serde_json::json!({"path": path_val})
//             ) {
//                 Ok(v) => v,
//                 Err(e) => {
//                     log(&format!("Failed to serialize args: {e:?}"));
//                     player.with_mut(|s| s.is_loading = false);
//                     return;
//                 }
//             };
//
//             log("Llamando a Tauri nativo...");
//
//             match tauri_invoke("play_file", args).await {
//                 Ok(result) => {
//                     log("tauri_invoke OK, convirtiendo JsValue...");
//
//                     // Convertir JsValue → serde_json::Value
//                     let json_value: Value = match serde_wasm_bindgen::from_value(result) {
//                         Ok(v) => v,
//                         Err(e) => {
//                             log(&format!("serde_wasm_bindgen::from_value FALLÓ: {e:?}"));
//                             player.with_mut(|s| s.is_loading = false);
//                             return;
//                         }
//                     };
//
//                     log(&format!("JSON Value recibido: {json_value:?}"));
//
//                     // Intentar parseo tipado
//                     match serde_json::from_value::<PlayResponse>(json_value.clone()) {
//                         Ok(resp) => {
//                             log(&format!("Parseo tipado OK: {resp:?}"));
//
//                             if let Some(meta) = resp.metadata {
//                                 log(&format!("Metadata — title: {}, artist: {}, duration: {}",
//                                              meta.title, meta.artist, meta.duration_secs));
//
//                                 player.with_mut(|s| {
//                                     s.metadata = Some(TrackMetadata {
//                                         title: meta.title,
//                                         artist: meta.artist,
//                                         duration_secs: meta.duration_secs,
//                                         path: path_val.clone(),
//                                     });
//                                     s.duration = meta.duration_secs;
//                                     s.current_time = 0.0;
//                                     s.is_playing = true;
//                                     s.is_loading = false;
//                                 });
//
//                                 log(&format!("Estado actualizado — duration: {}", player().duration));
//                             } else {
//                                 log("ERROR: metadata es None");
//                                 player.with_mut(|s| s.is_loading = false);
//                             }
//                         }
//                         Err(e_tipado) => {
//                             log(&format!("Parseo tipado FALLÓ: {e_tipado:?}"));
//                             log(&format!("Estructura raw: {json_value:?}"));
//
//                             // Fallback: leer manualmente desde Value
//                             if let Some(meta) = json_value.get("metadata") {
//                                 if let Some(dur) = meta.get("duration").and_then(serde_json::Value::as_f64) {
//                                     log(&format!("Fallback manual — duration: {dur}"));
//                                     player.with_mut(|s| {
//                                         s.duration = dur;
//                                         s.is_playing = true;
//                                         s.is_loading = false;
//                                     });
//                                 }
//                             } else {
//                                 player.with_mut(|s| s.is_loading = false);
//                             }
//                         }
//                     }
//                 }
//                 Err(e) => {
//                     log(&format!("tauri_invoke FALLÓ: {e:?}"));
//                     player.with_mut(|s| s.is_loading = false);
//                 }
//             }
//
//             log(&format!("=== PLAY finalizado — duration: {} ===", player().duration));
//         }
//     };
//
//     // ─── Handler pausar ───
//     let on_pause = move |_| {
//         let mut player = player;
//         async move {
//             let _ = tauri_invoke("pause", JsValue::NULL).await;
//             player.with_mut(|s| s.is_playing = false);
//         }
//     };
//
//     // ─── Handler reanudar ───
//     let on_resume = move |_| {
//         let mut player = player;
//         async move {
//             let _ = tauri_invoke("resume", JsValue::NULL).await;
//             player.with_mut(|s| s.is_playing = true);
//         }
//     };
//
//     // ─── Handler detener ───
//     let on_stop = move |_| {
//         let mut player = player;
//         async move {
//             let _ = tauri_invoke("stop", JsValue::NULL).await;
//             player.with_mut(|s| {
//                 s.is_playing = false;
//                 s.current_time = 0.0;
//             });
//         }
//     };
//
//     rsx! {
//         div { class: "test-audio",
//             input {
//                 value: "{path()}",
//                 oninput: move |e| path.set(e.value())
//             }
//             button {
//                 onclick: on_play,
//                 if player().is_loading {
//                     "⏳ Cargando..."
//                 } else {
//                     "▶ Reproducir"
//                 }
//             }
//             button {
//                 onclick: on_pause,
//                 "⏸ Pausar"
//             }
//             button {
//                 onclick: on_resume,
//                 "▶ Reanudar"
//             }
//             button {
//                 onclick: on_stop,
//                 "⏹ Detener"
//             }
//         }
//     }
// }

use chrono::Timelike;
use dioxus::prelude::*;

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
        // QuickAccessItem {
        //     title: "Podcast Diario",
        //     icon: "🎙",
        //     cover_url: None,
        // },
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

        let mut path = use_signal(|| {
            String::from(
                "C:\\Users\\PC\\Downloads\\J V N - You Got Me (Official Audio) [D1_686Kpj9w].opus",
            )
        });

        let player = use_player_state();

        fn log(msg: &str) {
            web_sys::console::log_1(&format!("[HOME] {msg}").into());
        }

        // ─── Handler reproducir ───
        let on_play = move |_: MouseEvent| {
            let path_val = path();
            let mut player = player;

            spawn(async move {
                log("=== PLAY iniciado ===");
                player.with_mut(|s| s.is_loading = true);

                // Serializar args con serde_json (escape automático del path)
                let args = match serde_wasm_bindgen::to_value(
                    &serde_json::json!({"path": path_val})
                ) {
                    Ok(v) => v,
                    Err(e) => {
                        log(&format!("Failed to serialize args: {e:?}"));
                        player.with_mut(|s| s.is_loading = false);
                        return;
                    }
                };

                log("Llamando a Tauri nativo...");

                match tauri_invoke("play_file", args).await {
                    Ok(result) => {
                        log("tauri_invoke OK, convirtiendo JsValue...");

                        // Convertir JsValue → serde_json::Value
                        let json_value: Value = match serde_wasm_bindgen::from_value(result) {
                            Ok(v) => v,
                            Err(e) => {
                                log(&format!("serde_wasm_bindgen::from_value FALLÓ: {e:?}"));
                                player.with_mut(|s| s.is_loading = false);
                                return;
                            }
                        };

                        log(&format!("JSON Value recibido: {json_value:?}"));

                        // Intentar parseo tipado
                        match serde_json::from_value::<PlayResponse>(json_value.clone()) {
                            Ok(resp) => {
                                log(&format!("Parseo tipado OK: {resp:?}"));

                                if let Some(meta) = resp.metadata {
                                    log(&format!("Metadata — title: {}, artist: {}, duration: {}",
                                                 meta.title, meta.artist, meta.duration_secs));

                                    player.with_mut(|s| {
                                        s.metadata = Some(TrackMetadata {
                                            title: meta.title,
                                            artist: meta.artist,
                                            duration_secs: meta.duration_secs,
                                            path: path_val.clone(),
                                            image: meta.image
                                        });
                                        s.duration = meta.duration_secs;
                                        s.current_time = 0.0;
                                        s.is_playing = true;
                                        s.is_loading = false;
                                    });

                                    log(&format!("Estado actualizado — duration: {}", player().duration));
                                } else {
                                    log("ERROR: metadata es None");
                                    player.with_mut(|s| s.is_loading = false);
                                }
                            }
                            Err(e_tipado) => {
                                log(&format!("Parseo tipado FALLÓ: {e_tipado:?}"));
                                log(&format!("Estructura raw: {json_value:?}"));

                                // Fallback: leer manualmente desde Value
                                if let Some(meta) = json_value.get("metadata") {
                                    if let Some(dur) = meta.get("duration").and_then(serde_json::Value::as_f64) {
                                        log(&format!("Fallback manual — duration: {dur}"));
                                        player.with_mut(|s| {
                                            s.duration = dur;
                                            s.is_playing = true;
                                            s.is_loading = false;
                                        });
                                    }
                                } else {
                                    player.with_mut(|s| s.is_loading = false);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log(&format!("tauri_invoke FALLÓ: {e:?}"));
                        player.with_mut(|s| s.is_loading = false);
                    }
                }

                log(&format!("=== PLAY finalizado — duration: {} ===", player().duration));
            });
        };

    quick[2] = QuickAccessItem {
        title: "Historial",
        icon: "🕐",
        cover_url: None,
        on_play: Some(EventHandler::new(on_play)),
    };

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

