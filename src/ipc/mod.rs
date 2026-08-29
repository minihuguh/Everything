pub mod commands;
pub mod dto;

use wasm_bindgen::prelude::*;
use web_sys::window;

pub(crate) fn log(msg: &str) {
    web_sys::console::log_1(&format!("[IPC] {msg}").into());
}

pub(crate) fn err(msg: &str) {
    web_sys::console::error_1(&format!("[IPC] {msg}").into());
}

fn get_invoke() -> Option<js_sys::Function> {
    let win = window()?;
    let tauri = js_sys::Reflect::get(&win, &"__TAURI__".into()).ok()?;
    if tauri.is_undefined() || tauri.is_null() {
        return None;
    }
    let core = js_sys::Reflect::get(&tauri, &"core".into()).ok();
    let invoke_target = core.as_ref().unwrap_or(&tauri);
    let invoke = js_sys::Reflect::get(invoke_target, &"invoke".into()).ok()?;
    if invoke.is_function() {
        Some(invoke.into())
    } else {
        None
    }
}

pub async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue> {
    let Some(invoke_fn) = get_invoke() else {
        log(&format!("runtime not available: {cmd}"));
        return Ok(JsValue::NULL);
    };
    let promise = invoke_fn.call2(&JsValue::NULL, &JsValue::from_str(cmd), &args)?;
    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise)).await
}
