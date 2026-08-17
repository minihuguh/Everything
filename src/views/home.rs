use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::window;
use serde::Deserialize;
use serde_json::Value;
use crate::state::{use_player_state, TrackMetadata};

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
}

#[component]
pub fn Home() -> Element {
    let mut path = use_signal(|| {
        String::from(
            "C:\\Users\\PC\\Downloads\\Alesso - Take My Breath Away (Lyric Video) [o10EV4PG40U].webm",
        )
    });

    let player = use_player_state();

    fn log(msg: &str) {
        web_sys::console::log_1(&format!("[HOME] {msg}").into());
    }

    // ─── Handler reproducir ───
    let on_play = move |_| {
        let path_val = path();
        let mut player = player;

        async move {
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
        }
    };

    // ─── Handler pausar ───
    let on_pause = move |_| {
        let mut player = player;
        async move {
            let _ = tauri_invoke("pause", JsValue::NULL).await;
            player.with_mut(|s| s.is_playing = false);
        }
    };

    // ─── Handler reanudar ───
    let on_resume = move |_| {
        let mut player = player;
        async move {
            let _ = tauri_invoke("resume", JsValue::NULL).await;
            player.with_mut(|s| s.is_playing = true);
        }
    };

    // ─── Handler detener ───
    let on_stop = move |_| {
        let mut player = player;
        async move {
            let _ = tauri_invoke("stop", JsValue::NULL).await;
            player.with_mut(|s| {
                s.is_playing = false;
                s.current_time = 0.0;
            });
        }
    };

    rsx! {
        div { class: "test-audio",
            input {
                value: "{path()}",
                oninput: move |e| path.set(e.value())
            }
            button {
                onclick: on_play,
                if player().is_loading {
                    "⏳ Cargando..."
                } else {
                    "▶ Reproducir"
                }
            }
            button {
                onclick: on_pause,
                "⏸ Pausar"
            }
            button {
                onclick: on_resume,
                "▶ Reanudar"
            }
            button {
                onclick: on_stop,
                "⏹ Detener"
            }
        }
    }
}