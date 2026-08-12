use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlElement, MouseEvent};
use dioxus::web::WebEventExt;
use std::cell::RefCell;
use std::rc::Rc;
use gloo_console::console;

// ─── Acceso seguro a Tauri via web_sys ───
fn get_tauri_invoke() -> Option<js_sys::Function> {
    let win = window()?;
    let tauri = js_sys::Reflect::get(&win, &"__TAURI__".into()).ok()?;

    if tauri.is_undefined() || tauri.is_null() {
        return None;
    }

    // En Tauri v2: window.__TAURI__.core.invoke
    // En Tauri v1: window.__TAURI__.invoke
    let core = js_sys::Reflect::get(&tauri, &"core".into()).ok();
    let invoke_target = core.as_ref().unwrap_or(&tauri);

    let invoke = js_sys::Reflect::get(invoke_target, &"invoke".into()).ok()?;

    if invoke.is_function() {
        Some(invoke.into())
    } else {
        None
    }
}

/// Invoca un comando de Tauri con fallback seguro
async fn tauri_invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue> {
    let invoke_fn = match get_tauri_invoke() {
        Some(f) => f,
        None => {
            web_sys::console::log_1(&format!("[TAURI NOT AVAILABLE] {}: {:?}", cmd, args).into());
            return Ok(JsValue::NULL);
        }
    };

    let cmd_js = JsValue::from_str(cmd);
    let this = JsValue::NULL;

    let promise = invoke_fn.call2(&this, &cmd_js, &args)?;
    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise)).await
}

// ─── Wrappers tipados ───
async fn tauri_set_volume(value: f64) {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "volume": value
    })).unwrap_or(JsValue::NULL);

    if let Err(e) = tauri_invoke("set_volume", args).await {
        web_sys::console::error_1(&format!("Error invoking set_volume: {:?}", e).into());
    }
}

async fn tauri_set_progress(value: f64) {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "progress": value
    })).unwrap_or(JsValue::NULL);

    if let Err(e) = tauri_invoke("set_progress", args).await {
        web_sys::console::error_1(&format!("Error invoking set_progress: {:?}", e).into());
    }
}

// ─── Debounce ───
#[derive(Clone)]
struct DebouncedSender {
    inner: Rc<RefCell<DebouncedInner>>,
}

struct DebouncedInner {
    timeout_id: Option<i32>,
    delay_ms: i32,
    callback: Box<dyn Fn(f64)>,
}

impl DebouncedSender {
    fn new<F>(callback: F, delay_ms: i32) -> Self
    where
        F: Fn(f64) + 'static,
    {
        Self {
            inner: Rc::new(RefCell::new(DebouncedInner {
                timeout_id: None,
                delay_ms,
                callback: Box::new(callback),
            })),
        }
    }

    fn send(&self, value: f64) {
        let mut inner = self.inner.borrow_mut();

        if let Some(id) = inner.timeout_id.take() {
            let _ = window().unwrap().clear_timeout_with_handle(id);
        }

        let self_clone = self.clone();
        let closure = Closure::once_into_js(move || {
            self_clone.fire(value);
        });

        let new_id = window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                inner.delay_ms,
            )
            .unwrap();

        inner.timeout_id = Some(new_id);
    }

    fn fire(&self, value: f64) {
        let inner = self.inner.borrow();
        (inner.callback)(value);
        drop(inner);
        self.inner.borrow_mut().timeout_id = None;
    }
}

//noinspection D
#[component]
pub fn PlayerBar() -> Element {
    let mut is_playing = use_signal(|| false);
    let mut progress = use_signal(|| 0.0_f64);
    let mut volume = use_signal(|| 70.0_f64);

    let mut dragging_vol = use_signal(|| false);
    let mut dragging_prog = use_signal(|| false);

    let mut vol_el: Signal<Option<HtmlElement>> = use_signal(|| None);
    let mut prog_el: Signal<Option<HtmlElement>> = use_signal(|| None);

    // ─── Senders debounced ───
    let send_vol = use_hook(|| {
        DebouncedSender::new(move |value: f64| {
            web_sys::console::log_1(&format!("[VOLUMEN] {:?}",value.to_string()).into());
            wasm_bindgen_futures::spawn_local(async move {
                tauri_set_volume(value / 100.0).await;
            });
        }, 50)
    });

    let send_prog = use_hook(|| {
        DebouncedSender::new(move |value: f64| {
            wasm_bindgen_futures::spawn_local(async move {
                tauri_set_progress(value).await;
            });
        }, 100)
    });

    fn calc_pct(el: &HtmlElement, client_x: f64) -> f64 {
        let rect = el.get_bounding_client_rect();
        let w = rect.width();
        if w > 0.0 {
            ((client_x - rect.left()) / w * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        }
    }

    use_effect(move || {
        let is_dragging = dragging_vol() || dragging_prog();
        if !is_dragging {
            return;
        }

        let win = window().expect("window");
        let doc = win.document().expect("document");

        let vol_el = vol_el;
        let prog_el = prog_el;
        let mut volume = volume;
        let mut progress = progress;
        let mut dragging_vol = dragging_vol;
        let mut dragging_prog = dragging_prog;

        let on_move = Closure::<dyn FnMut(MouseEvent)>::new(move |e: MouseEvent| {
            let x = e.client_x() as f64;
            if dragging_vol() {
                if let Some(el) = vol_el() {
                    volume.set(calc_pct(&el, x));
                }
            }
            if dragging_prog() {
                if let Some(el) = prog_el() {
                    progress.set(calc_pct(&el, x));
                }
            }
        });

        let on_up = Closure::<dyn FnMut()>::new(move || {
            dragging_vol.set(false);
            dragging_prog.set(false);
        });

        doc.add_event_listener_with_callback("mousemove", on_move.as_ref().unchecked_ref())
            .unwrap();
        doc.add_event_listener_with_callback("mouseup", on_up.as_ref().unchecked_ref())
            .unwrap();

        on_move.forget();
        on_up.forget();
    });

    use_effect(move || {
        send_vol.send(volume());
    });

    use_effect(move || {
        send_prog.send(progress());
    });

    let on_vol_scroll = use_callback(move |e: Event<WheelData>| {
        e.prevent_default();
        let delta_y = e.as_web_event()
            .dyn_ref::<web_sys::WheelEvent>()
            .map(|we| we.delta_y())
            .unwrap_or(0.0);

        let step = 5.0;
        let current = volume();
        let new_vol = if delta_y > 0.0 {
            (current - step).max(0.0)
        } else if delta_y < 0.0 {
            (current + step).min(100.0)
        } else {
            current
        };

        if (new_vol - current).abs() > 0.01 {
            volume.set(new_vol);
        }
    });

    let on_vol_down = use_callback(move |e: Event<MouseData>| {
        e.prevent_default();
        dragging_vol.set(true);
        if let Some(el) = vol_el() {
            let coords = e.data.client_coordinates();
            volume.set(calc_pct(&el, coords.x));
        }
    });

    let on_prog_down = use_callback(move |e: Event<MouseData>| {
        e.prevent_default();
        dragging_prog.set(true);
        if let Some(el) = prog_el() {
            let coords = e.data.client_coordinates();
            progress.set(calc_pct(&el, coords.x));
        }
    });

    let vol_icon = use_memo(move || match volume() {
        v if v == 0.0 => "🔇",
        v if v < 30.0 => "🔈",
        v if v < 70.0 => "🔉",
        _ => "🔊",
    });

    let progress_width = use_memo(move || format!("{:.1}", progress()));
    let progress_left = use_memo(move || format!("{:.1}", progress()));
    let volume_width = use_memo(move || format!("{:.1}", volume()));
    let volume_left = use_memo(move || format!("{:.1}", volume()));

    let is_dragging_vol = dragging_vol();
    let is_dragging_prog = dragging_prog();

    rsx! {
        div { class: "player-bar",
            div { class: "player-info",
                div { class: "player-cover",
                    img {
                        src: asset!("assets/cover.png"),
                        alt: "cover"
                    }
                }
                div { class: "player-text",
                    div { class: "player-title", "Every breath you take" }
                    div { class: "player-artist", "J V N" }
                }
            }

            div { class: "player-center",
                div { class: "player-progress-row",
                    span { class: "time", "00:00" }
                    div {
                        class: "slider-host progress-slider",
                        onmounted: move |evt: Event<MountedData>| {
                            if let Ok(el) = evt.as_web_event().dyn_into::<HtmlElement>() {
                                prog_el.set(Some(el));
                            }
                        },
                        onmousedown: on_prog_down,
                        div {
                            class: "slider-fill",
                            style: "width: {progress_width}%"
                        }
                        div {
                            class: "slider-knob",
                            class: if is_dragging_prog { "dragging" } else { "" },
                            style: "left: {progress_left}%"
                        }
                    }
                    span { class: "time", "02:55" }
                }

                div { class: "player-buttons",
                    button { class: "player-btn",
                        svg { width: "24", height: "24", view_box: "0 0 24 24", fill: "currentColor",
                            path { d: "M6 6h2v12H6zm3.5 6l8.5 6V6z" }
                        }
                    }
                    button { class: "player-btn",
                        svg { width: "24", height: "24", view_box: "0 0 24 24", fill: "currentColor",
                            path { d: "M11 18V6l-8.5 6 8.5 6zm.5-6l8.5 6V6l-8.5 6z" }
                        }
                    }
                    button {
                        class: "player-btn play",
                        onclick: move |_| is_playing.set(!is_playing()),
                        if is_playing() {
                            svg { width: "28", height: "28", view_box: "0 0 24 24", fill: "currentColor",
                                path { d: "M6 19h4V5H6v14zm8-14v14h4V5h-4z" }
                            }
                        } else {
                            svg { width: "28", height: "28", view_box: "0 0 24 24", fill: "currentColor",
                                path { d: "M8 5v14l11-7z" }
                            }
                        }
                    }
                    button { class: "player-btn",
                        svg { width: "24", height: "24", view_box: "0 0 24 24", fill: "currentColor",
                            path { d: "M4 18l8.5-6L4 6v12zm9-12v12l8.5-6L13 6z" }
                        }
                    }
                    button { class: "player-btn",
                        svg { width: "24", height: "24", view_box: "0 0 24 24", fill: "currentColor",
                            path { d: "M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z" }
                        }
                    }
                }
            }

            div { class: "player-volume",
                span { class: "vol-icon", {vol_icon} }

                div {
                    class: "slider-host volume-slider",
                    onmounted: move |evt: Event<MountedData>| {
                        if let Ok(el) = evt.as_web_event().dyn_into::<HtmlElement>() {
                            vol_el.set(Some(el));
                        }
                    },
                    onmousedown: on_vol_down,
                    onwheel: on_vol_scroll,
                    div {
                        class: "slider-fill",
                        style: "width: {volume_width}%"
                    }
                    div {
                        class: "slider-knob",
                        class: if is_dragging_vol { "dragging" } else { "" },
                        style: "left: {volume_left}%"
                    }
                }

                button { class: "player-btn",
                    svg { width: "20", height: "20", view_box: "0 0 24 24", fill: "currentColor",
                        path { d: "M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z" }
                    }
                }
                button { class: "player-btn",
                    svg { width: "20", height: "20", view_box: "0 0 24 24", fill: "currentColor",
                        path { d: "M11.99 2C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zM12 20c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8zm.5-13H11v6l5.25 3.15.75-1.23-4.5-2.67z" }
                    }
                }
                button { class: "player-btn",
                    svg { width: "20", height: "20", view_box: "0 0 24 24", fill: "currentColor",
                        path { d: "M21 3H3c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h18c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 13H3V5h18v11z" }
                    }
                }
            }
        }
    }
}