use dioxus::prelude::*;
use dioxus::document::eval;
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlElement};
use std::cell::RefCell;
use std::rc::Rc;
use dioxus::web::WebEventExt;
use crate::state::{use_player_state, RepeatMode};

fn log(msg: &str) {
    web_sys::console::log_1(&format!("[PLAYER_BAR] {msg}").into());
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

async fn tauri_invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue> {
    let Some(invoke_fn) = get_tauri_invoke() else {
        web_sys::console::log_1(&format!("[TAURI NOT AVAILABLE] {cmd}").into());
        return Ok(JsValue::NULL);
    };
    let cmd_js = JsValue::from_str(cmd);
    let this = JsValue::NULL;
    let promise = invoke_fn.call2(&this, &cmd_js, &args)?;
    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise)).await
}

async fn tauri_get_time() -> Option<f64> {
    match tauri_invoke("get_time", JsValue::NULL).await {
        Ok(val) => val.as_f64(),
        Err(e) => {
            web_sys::console::error_1(&format!("get_time error: {e:?}").into());
            None
        }
    }
}

async fn tauri_set_time(secs: f64) {
    let Ok(args) = serde_wasm_bindgen::to_value(
        &serde_json::json!({"timeInSeconds": secs.round() as u64})
    ) else {
        web_sys::console::error_1(&"Failed to serialize set_time args".into());
        return;
    };
    if let Err(e) = tauri_invoke("set_time", args).await {
        web_sys::console::error_1(&format!("set_time error: {e:?}").into());
    }
}

async fn tauri_set_volume(value: f64) {
    let Ok(args) = serde_wasm_bindgen::to_value(
        &serde_json::json!({"volume": value})
    ) else { return };
    if let Err(e) = tauri_invoke("set_volume", args).await {
        web_sys::console::error_1(&format!("set_volume error: {e:?}").into());
    }
}

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
            let () = window().unwrap().clear_timeout_with_handle(id);
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

#[component]
pub fn PlayerBar(show_queue: Signal<bool>) -> Element {
    let mut player = use_player_state();

    let is_playing_memo = use_memo(move || player().is_playing);
    let volume_memo = use_memo(move || player().volume);

    let mut dragging_vol = use_signal(|| false);
    let mut dragging_prog = use_signal(|| false);
    let mut vol_el: Signal<Option<HtmlElement>> = use_signal(|| None);
    let mut prog_el: Signal<Option<HtmlElement>> = use_signal(|| None);

    let interval_id: Rc<RefCell<Option<i32>>> = use_hook(|| Rc::new(RefCell::new(None)));

    let should_poll = use_memo(move || {
        is_playing_memo() && !dragging_prog()
    });

    let interval_id_for_effect = interval_id.clone();
    use_effect(move || {
        let should = should_poll();
        log(&format!("Efecto — should_poll={should}"));

        {
            let mut guard = interval_id_for_effect.borrow_mut();
            if let Some(id) = guard.take() {
                log(&format!("Limpiando intervalo anterior: {id}"));
                let () = window().unwrap().clear_interval_with_handle(id);
            }
        }

        if !should {
            log("No se crea intervalo");
            return;
        }

        log("Creando nuevo intervalo de 1s...");
        let player = player;

        let closure = Closure::wrap(Box::new(move || {
            let mut player = player;
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(time) = tauri_get_time().await {
                    player.with_mut(|p| {
                        p.current_time = time;
                        if p.duration > 0.0 && time >= p.duration - 1.0 {
                            p.advance();
                            log(&format!("Cancion finalizada — avanzando a siguiente"));
                        }
                    });
                }
            });
        }) as Box<dyn FnMut()>);

        let id = window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                1000,
            )
            .unwrap();

        *interval_id_for_effect.borrow_mut() = Some(id);
        log(&format!("Intervalo creado: {id}"));
        closure.forget();
    });

    use_effect(move || {
        let vol = player().volume / 100.0;
        wasm_bindgen_futures::spawn_local(async move {
            tauri_set_volume(vol).await;
        });
    });

    let send_vol = use_hook(|| {
        DebouncedSender::new(move |value: f64| {
            wasm_bindgen_futures::spawn_local(async move {
                tauri_set_volume(value / 100.0).await;
            });
        }, 50)
    });

    fn calc_pct(el: &HtmlElement, client_x: f64) -> f64 {
        let rect = el.get_bounding_client_rect();
        let w = rect.width();
        if w > 0.0 {
            ((client_x - rect.left()) / w).clamp(0.0, 1.0)
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
        let mut player = player;
        let mut dragging_vol = dragging_vol;
        let mut dragging_prog = dragging_prog;

        let on_move = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |e: web_sys::MouseEvent| {
            let x = f64::from(e.client_x());
            if dragging_vol()
                && let Some(el) = vol_el() {
                let pct = calc_pct(&el, x);
                player.with_mut(|p| p.volume = pct * 100.0);
            }
            if dragging_prog()
                && let Some(el) = prog_el() {
                let pct = calc_pct(&el, x);
                let duration = player().duration;
                if duration > 0.0 {
                    player.with_mut(|p| p.current_time = pct * duration);
                }
            }
        });

        let on_up = Closure::<dyn FnMut()>::new(move || {
            let was_dragging_prog = dragging_prog();
            dragging_vol.set(false);
            dragging_prog.set(false);

            if was_dragging_prog {
                let time = player().current_time;
                log(&format!("Mouse up — enviando set_time({time})"));
                wasm_bindgen_futures::spawn_local(async move {
                    tauri_set_time(time).await;
                });
            }
        });

        doc.add_event_listener_with_callback("mousemove", on_move.as_ref().unchecked_ref())
            .unwrap();
        doc.add_event_listener_with_callback("mouseup", on_up.as_ref().unchecked_ref())
            .unwrap();

        on_move.forget();
        on_up.forget();
    });

    use_effect(move || {
        if !dragging_vol() {
            send_vol.send(volume_memo());
        }
    });

    let on_vol_scroll = use_callback(move |e: Event<WheelData>| {
        e.prevent_default();
        let delta_y = e.as_web_event()
            .dyn_ref::<web_sys::WheelEvent>()
            .map_or(0.0, web_sys::WheelEvent::delta_y);

        let step = 5.0;
        let current = player().volume;
        let new_vol = if delta_y > 0.0 {
            (current - step).max(0.0)
        } else if delta_y < 0.0 {
            (current + step).min(100.0)
        } else {
            current
        };

        if (new_vol - current).abs() > 0.01 {
            player.with_mut(|p| p.volume = new_vol);
        }
    });

    let on_vol_down = use_callback(move |e: Event<MouseData>| {
        e.prevent_default();
        dragging_vol.set(true);
        if let Some(el) = vol_el() {
            let coords = e.data.client_coordinates();
            let pct = calc_pct(&el, coords.x);
            player.with_mut(|p| p.volume = pct * 100.0);
        }
    });

    let on_prog_down = use_callback(move |e: Event<MouseData>| {
        e.prevent_default();
        log("Mouse down en progreso — dragging_prog = true");
        dragging_prog.set(true);
        if let Some(el) = prog_el() {
            let coords = e.data.client_coordinates();
            let pct = calc_pct(&el, coords.x);
            let duration = player().duration;
            log(&format!("Click en progreso — pct={pct}, duration={duration}"));
            if duration > 0.0 {
                let new_time = pct * duration;
                player.with_mut(|p| p.current_time = new_time);
                log(&format!("current_time actualizado a: {new_time}"));
            }
        }
    });

    let on_prog_scroll = use_callback(move |e: Event<WheelData>| {
        e.prevent_default();
        let delta_y = e.as_web_event()
            .dyn_ref::<web_sys::WheelEvent>()
            .map_or(0.0, web_sys::WheelEvent::delta_y);

        let duration = player().duration;
        if duration <= 0.0 { return; }

        let step_secs = 5.0;
        let current = player().current_time;

        let new_time = if delta_y > 0.0 {
            (current - step_secs).max(0.0)
        } else if delta_y < 0.0 {
            (current + step_secs).min(duration)
        } else {
            current
        };

        if (new_time - current).abs() > 0.01 {
            player.with_mut(|p| p.current_time = new_time);
            wasm_bindgen_futures::spawn_local(async move {
                tauri_set_time(new_time).await;
            });
        }
    });

    let on_toggle_play = move |_| {
        let is_playing_now = player().is_playing;
        let mut player = player;
        let cmd = if is_playing_now { "pause" } else { "resume" };
        async move {
            let _ = eval(&format!("window.__TAURI__.core.invoke('{cmd}')")).await;
            player.with_mut(|p| p.is_playing = !is_playing_now);
        }
    };

    let on_prev = move |_| {
        let mut player = player;
        async move {
            player.with_mut(|p| p.go_back());
            if let Some(track) = player().current_track() {
                let path = track.path.clone();
                let args = match serde_wasm_bindgen::to_value(&serde_json::json!({"path": path})) {
                    Ok(v) => v,
                    Err(_) => return,
                };
                let _ = tauri_invoke("play_file", args).await;
            }
        }
    };

    let on_next = move |_| {
        let mut player = player;
        async move {
            player.with_mut(|p| p.advance());
            if let Some(track) = player().current_track() {
                let path = track.path.clone();
                let args = match serde_wasm_bindgen::to_value(&serde_json::json!({"path": path})) {
                    Ok(v) => v,
                    Err(_) => return,
                };
                let _ = tauri_invoke("play_file", args).await;
            }
        }
    };

    let on_repeat = move |_| {
        player.with_mut(|p| p.toggle_repeat());
    };

    let on_shuffle = move |_| {
        player.with_mut(|p| p.toggle_shuffle());
    };

    let progress_pct = use_memo(move || player().progress_pct());
    let volume_pct = use_memo(move || player().volume);
    let current_time_str = use_memo(move || player().format_time());
    let duration_str = use_memo(move || player().format_duration());

    let title = use_memo(move || {
        player()
            .metadata
            .as_ref().map_or_else(|| "Sin reproduccion".to_string(), |m| m.title.clone())
    });

    let artist = use_memo(move || {
        player()
            .metadata
            .as_ref().map_or_else(|| "—".to_string(), |m| m.artist.clone())
    });

    let image = use_memo(move || {
        player()
            .metadata
            .as_ref().map_or_else(|| "Null".to_string(), |m| m.image.clone())
    });

    let repeat_mode = use_memo(move || player().repeat_mode.clone());
    let is_shuffled = use_memo(move || player().is_shuffled);
    let has_prev = use_memo(move || player().prev_track_exists());
    let has_next = use_memo(move || player().next_track_exists());
    let is_playing = player().is_playing;
    let is_dragging_vol = dragging_vol();
    let is_dragging_prog = dragging_prog();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/player_bar.css") }

        div { class: "player-bar",
            div { class: "player-info",
                div { class: if image() != "Null" { "player-cover" } else { "player-cover empty" },
                    if image() != "Null" {
                        img {
                            src: "data:image/jpeg;base64,{image}",
                            alt: "cover"
                        }
                    }
                }
                div { class: "player-text",
                    div { class: "player-title", "{title}" }
                    div { class: "player-artist", "{artist}" }
                }
            }

            div { class: "player-center",
                div { class: "player-progress-row",
                    span { class: "time", "{current_time_str}" }
                    div {
                        class: "slider-host progress-slider",
                        onmounted: move |evt: Event<MountedData>| {
                            if let Ok(el) = evt.as_web_event().dyn_into::<HtmlElement>() {
                                prog_el.set(Some(el));
                            }
                        },
                        onmousedown: on_prog_down,
                        onwheel: on_prog_scroll,
                        div {
                            class: "slider-fill",
                            style: "width: {progress_pct}%"
                        }
                        div {
                            class: "slider-knob",
                            class: if is_dragging_prog { "dragging" } else { "" },
                            style: "left: {progress_pct}%"
                        }
                    }
                    span { class: "time", "{duration_str}" }
                }

                div { class: "player-buttons",
                    button {
                        class: if is_shuffled() { "player-btn active" } else { "player-btn" },
                        onclick: on_shuffle,
                        svg { width: "20", height: "20", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round",
                            polyline { points: "16 3 21 3 21 8" }
                            line { x1: "4", y1: "20", x2: "21", y2: "3", }
                            polyline { points: "21 16 21 21 16 21" }
                            line { x1: "15", y1: "15", x2: "21", y2: "21", }
                            line { x1: "4", y1: "4", x2: "9", y2: "9", }
                        }
                    }
                    button {
                        class: if has_prev() { "player-btn" } else { "player-btn disabled" },
                        onclick: on_prev,
                        svg { width: "24", height: "24", view_box: "0 0 24 24", fill: "currentColor",
                            path { d: "M11 18V6l-8.5 6 8.5 6zm.5-6l8.5 6V6l-8.5 6z" }
                        }
                    }
                    button {
                        class: "player-btn play",
                        onclick: on_toggle_play,
                        if is_playing {
                            svg { width: "28", height: "28", view_box: "0 0 24 24", fill: "currentColor",
                                path { d: "M6 19h4V5H6v14zm8-14v14h4V5h-4z" }
                            }
                        } else {
                            svg { width: "28", height: "28", view_box: "0 0 24 24", fill: "currentColor",
                                path { d: "M8 5v14l11-7z" }
                            }
                        }
                    }
                    button {
                        class: if has_next() { "player-btn" } else { "player-btn disabled" },
                        onclick: on_next,
                        svg { width: "24", height: "24", view_box: "0 0 24 24", fill: "currentColor",
                            path { d: "M4 18l8.5-6L4 6v12zm9-12v12l8.5-6L13 6z" }
                        }
                    }
                    button {
                        class: match repeat_mode() {
                            RepeatMode::Off => "player-btn",
                            _ => "player-btn active",
                        },
                        onclick: on_repeat,
                        svg { width: "20", height: "20", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round",
                            polyline { points: "17 1 21 5 17 9" }
                            path { d: "M3 11V9a4 4 0 0 1 4-4h14" }
                            polyline { points: "7 23 3 19 7 15" }
                            path { d: "M21 13v2a4 4 0 0 1-4 4H3" }
                            if repeat_mode() == RepeatMode::One {
                                text { x: "10", y: "17", fill: "currentColor", font_size: "10", font_weight: "bold", "1" }
                            }
                        }
                    }
                }
            }

            div { class: "player-volume",
                if volume_pct() == 0.0 {
                    svg { width: "20", height: "20", view_box: "0 0 24 24", fill: "none", stroke: "#a0a0b0", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round",
                        polygon { points: "11 5 6 9 2 9 2 15 6 15 11 19 11 5" }
                        line { x1: "23", y1: "9", x2: "17", y2: "15" }
                        line { x1: "17", y1: "9", x2: "23", y2: "15" }
                    }
                } else if volume_pct() < 30.0 {
                    svg { width: "20", height: "20", view_box: "0 0 24 24", fill: "none", stroke: "#a0a0b0", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round",
                        polygon { points: "11 5 6 9 2 9 2 15 6 15 11 19 11 5" }
                    }
                } else if volume_pct() < 70.0 {
                    svg { width: "20", height: "20", view_box: "0 0 24 24", fill: "none", stroke: "#a0a0b0", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round",
                        polygon { points: "11 5 6 9 2 9 2 15 6 15 11 19 11 5" }
                        path { d: "M15.54 8.46a5 5 0 0 1 0 7.07" }
                    }
                } else {
                    svg { width: "20", height: "20", view_box: "0 0 24 24", fill: "none", stroke: "#a0a0b0", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round",
                        polygon { points: "11 5 6 9 2 9 2 15 6 15 11 19 11 5" }
                        path { d: "M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07" }
                    }
                }

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
                        style: "width: {volume_pct}%"
                    }
                    div {
                        class: "slider-knob",
                        class: if is_dragging_vol { "dragging" } else { "" },
                        style: "left: {volume_pct}%"
                    }
                }

                button {
                    class: "player-btn",
                    onclick: move |_| show_queue.set(!show_queue()),
                    svg { width: "20", height: "20", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round",
                        path { d: "M4 6h16M4 12h16M4 18h16" }
                    }
                }
            }
        }
    }
}