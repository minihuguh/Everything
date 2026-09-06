use super::dto::{Metadata, PlayResponse};
use super::{err, invoke, log};
use wasm_bindgen::prelude::*;

fn args(value: serde_json::Value) -> Option<JsValue> {
    serde_wasm_bindgen::to_value(&value)
        .map_err(|e| err(&format!("args serialization: {e:?}")))
        .ok()
}

pub async fn get_time() -> Option<f64> {
    match invoke("get_time", JsValue::NULL).await {
        Ok(v) => v.as_f64(),
        Err(e) => {
            err(&format!("get_time: {e:?}"));
            None
        }
    }
}

pub async fn set_time(secs: f64) {
    let Some(args) = args(serde_json::json!({ "timeInSeconds": secs.round() as u64 })) else {
        return;
    };
    if let Err(e) = invoke("set_time", args).await {
        err(&format!("set_time: {e:?}"));
    }
}

pub async fn set_volume(fraction: f64) {
    let Some(args) = args(serde_json::json!({ "volume": fraction })) else {
        return;
    };
    if let Err(e) = invoke("set_volume", args).await {
        err(&format!("set_volume: {e:?}"));
    }
}

pub async fn playlist() {
    match invoke("open_playlist", JsValue::UNDEFINED).await {
        Ok(v) => {
            // let playlist = Playlist::read_file(&v.as_string().unwrap());
            log(&format!("playlist: {v:?}"));
        },
        Err(e) => {
            err(&format!("select_document: {e:?}"));
        }
    }

}

pub async fn play_file(path: &str) -> Option<PlayResponse> {
    let args = args(serde_json::json!({ "path": path }))?;
    let raw = match invoke("play_file", args).await {
        Ok(v) => v,
        Err(e) => {
            err(&format!("play_file: {e:?}"));
            return None;
        }
    };
    match serde_wasm_bindgen::from_value::<serde_json::Value>(raw) {
        Ok(json) => serde_json::from_value::<PlayResponse>(json.clone())
            .map_err(|e| err(&format!("play_file: parse failed ({e:?}) — raw={json:?}")))
            .ok(),
        Err(e) => {
            err(&format!("play_file: JsValue -> Value failed: {e:?}"));
            None
        }
    }
}

pub async fn get_metadata(path: &str) -> Option<Metadata> {
    let args = args(serde_json::json!({ "path": path }))?;
    let raw = match invoke("get_metadata", args).await {
        Ok(v) => v,
        Err(e) => {
            err(&format!("metadata: {e:?}"));
            return None;
        }
    };
    match serde_wasm_bindgen::from_value::<serde_json::Value>(raw) {
        Ok(json) => serde_json::from_value::<Metadata>(json.clone())
            .map_err(|e| err(&format!("metadata: parse failed ({e:?}) — raw={json:?}")))
            .ok(),
        Err(e) => {
            err(&format!("metadata: JsValue -> Value failed: {e:?}"));
            None
        }
    }
}

pub async fn pause() {
    let _ = invoke("pause", JsValue::NULL).await;
}

pub async fn resume() {
    let _ = invoke("resume", JsValue::NULL).await;
}

#[allow(dead_code)]
pub async fn stop() {
    let _ = invoke("stop", JsValue::NULL).await;
}

pub async fn select_document() -> Option<String> {
    match invoke("select_document", JsValue::UNDEFINED).await {
        Ok(v) => v.as_string(),
        Err(e) => {
            err(&format!("select_document: {e:?}"));
            None
        }
    }
}

pub async fn minimize_window() {
    let _ = invoke("minimize_window", JsValue::NULL).await;
}

pub async fn toggle_maximize() {
    let _ = invoke("toggle_maximize", JsValue::NULL).await;
}

pub async fn close_window() {
    let _ = invoke("close_window", JsValue::NULL).await;
}
