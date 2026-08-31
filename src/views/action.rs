use dioxus::prelude::*;
use serde::Deserialize;
use serde_json::Value;
use wasm_bindgen::JsValue;
use web_sys::window;
use crate::state::{PlayerState, TrackMetadata};

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

pub async fn tauri_invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue> {
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

pub fn log(msg: &str) {
        web_sys::console::log_1(&format!("[HOME] {msg}").into());
    }

pub fn reproducir_archivo_global(path_val: String, mut player: Signal<PlayerState>) {
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

                let json_value: Value = match serde_wasm_bindgen::from_value(result) {
                    Ok(v) => v,
                    Err(e) => {
                        log(&format!("serde_wasm_bindgen::from_value FALLÓ: {e:?}"));
                        player.with_mut(|s| s.is_loading = false);
                        return;
                    }
                };

                // log(&format!("JSON Value recibido: {json_value:?}"));

                match serde_json::from_value::<PlayResponse>(json_value.clone()) {
                    Ok(resp) => {
                        // log(&format!("Parseo tipado OK: {resp:?}"));

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
}
