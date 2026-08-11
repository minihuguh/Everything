use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlElement, MouseEvent};
use gloo_console::log as console_log;
use dioxus::web::WebEventExt;

#[component]
pub fn PlayerBar() -> Element {
    let mut is_playing = use_signal(|| false);
    let mut progress = use_signal(|| 0.0_f64);
    let mut volume = use_signal(|| 70.0_f64); // 0-100 para facilitar

    let mut dragging_vol = use_signal(|| false);
    let mut dragging_prog = use_signal(|| false);

    let mut vol_el: Signal<Option<HtmlElement>> = use_signal(|| None);
    let mut prog_el: Signal<Option<HtmlElement>> = use_signal(|| None);

    fn calc_pct(el: &HtmlElement, client_x: f64) -> f64 {
        let rect = el.get_bounding_client_rect();
        let w = rect.width();
        let left = rect.left();
        let pct = if w > 0.0 {
            ((client_x - left) / w * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };
        console_log!(format!("calc_pct: client_x={}, left={}, width={}, pct={:.2}", client_x, left, w, pct));
        pct
    }

    // Efecto drag global
    use_effect(move || {
        let is_dragging_vol = dragging_vol();
        let is_dragging_prog = dragging_prog();

        console_log!(format!("use_effect triggered: vol={}, prog={}", is_dragging_vol, is_dragging_prog));

        if !is_dragging_vol && !is_dragging_prog {
            return;
        }

        let win = window().expect("window");
        let doc = win.document().expect("document");

        let on_move = Closure::<dyn FnMut(MouseEvent)>::new({
            let vol_el = vol_el;
            let prog_el = prog_el;
            let mut volume = volume;
            let mut progress = progress;
            let dragging_vol = dragging_vol;
            let dragging_prog = dragging_prog;

            move |e: MouseEvent| {
                let x = e.client_x() as f64;
                if dragging_vol() {
                    if let Some(el) = vol_el() {
                        let new_vol = calc_pct(&el, x);
                        volume.set(new_vol);
                        console_log!(format!("DRAG VOL: x={}, new_vol={:.2}", x, new_vol));
                    } else {
                        console_log!("DRAG VOL: vol_el is None!");
                    }
                }
                if dragging_prog() {
                    if let Some(el) = prog_el() {
                        let new_prog = calc_pct(&el, x);
                        progress.set(new_prog);
                        console_log!(format!("DRAG PROG: x={}, new_prog={:.2}", x, new_prog));
                    }
                }
            }
        });

        let on_up = Closure::<dyn FnMut()>::new({
            let mut dragging_vol = dragging_vol;
            let mut dragging_prog = dragging_prog;
            move || {
                console_log!("MOUSEUP: stopping drag");
                dragging_vol.set(false);
                dragging_prog.set(false);
            }
        });

        doc.add_event_listener_with_callback("mousemove", on_move.as_ref().unchecked_ref())
            .unwrap();
        doc.add_event_listener_with_callback("mouseup", on_up.as_ref().unchecked_ref())
            .unwrap();

        on_move.forget();
        on_up.forget();
    });

    let on_vol_down = move |e: Event<MouseData>| {
        e.prevent_default();
        console_log!("VOLUME MOUSEDOWN");
        dragging_vol.set(true);
        if let Some(el) = vol_el() {
            let coords = e.data.client_coordinates();
            let new_vol = calc_pct(&el, coords.x);
            volume.set(new_vol);
            console_log!(format!("VOL CLICK: coords.x={}, new_vol={:.2}", coords.x, new_vol));
        } else {
            console_log!("VOL MOUSEDOWN: vol_el is None!");
        }
    };

    let on_prog_down = move |e: Event<MouseData>| {
        e.prevent_default();
        console_log!("PROGRESS MOUSEDOWN");
        dragging_prog.set(true);
        if let Some(el) = prog_el() {
            let coords = e.data.client_coordinates();
            let new_prog = calc_pct(&el, coords.x);
            progress.set(new_prog);
            console_log!(format!("PROG CLICK: coords.x={}, new_prog={:.2}", coords.x, new_prog));
        }
    };

    // Icono de volumen dinámico
    let vol_icon = move || match volume() {
        v if v == 0.0 => "🔇",
        v if v < 30.0 => "🔈",
        v if v < 70.0 => "🔉",
        _ => "🔊",
    };

    rsx! {
        div { class: "player-bar",
            // Info izquierda
            div { class: "player-info",
                div { class: "player-cover",
                    img {
                        src: asset!("assets/cover.png"),
                        alt: "cover"
                    }
                }
                div { class: "player-text",
                    div { class: "player-title", "Every breath you take" }
                    // DEBUG: muestra volumen en lugar del artista
                    div { class: "player-artist", "J V N" }
                }
            }

            // Controles centro
            div { class: "player-center",
                // Barra de progreso
                div { class: "player-progress-row",
                    span { class: "time", "00:00" }
                    div {
                        class: "slider-host progress-slider",
                        onmounted: move |evt: Event<MountedData>| {
                            if let Ok(el) = evt.as_web_event().dyn_into::<HtmlElement>() {
                                let rect = el.get_bounding_client_rect();
                                console_log!(format!(
                                    "PROG MOUNTED: width={}, left={}, top={}",
                                    rect.width(), rect.left(), rect.top()
                                ));
                                prog_el.set(Some(el));
                            } else {
                                console_log!("PROG MOUNTED: FAILED!");
                            }
                        },
                        onmousedown: on_prog_down,
                        // DEBUG: fill rojo para ver cambios
                        div {
                            class: "slider-fill debug-fill",
                            style: "width: {progress:.1}%"
                        }
                        div {
                            class: "slider-knob",
                            class: if dragging_prog() { "dragging" } else { "" },
                            style: "left: {progress:.1}%"
                        }
                    }
                    span { class: "time", "{progress:.0}%" } // DEBUG: muestra %
                }

                // Botones
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

            // Volumen derecha
            div { class: "player-volume",
                span { class: "vol-icon", {vol_icon()} }

                div {
                    class: "slider-host volume-slider",
                    onmounted: move |evt: Event<MountedData>| {
                        if let Ok(el) = evt.as_web_event().dyn_into::<HtmlElement>() {
                            let rect = el.get_bounding_client_rect();
                            console_log!(format!(
                                "VOL MOUNTED: width={}, left={}, top={}",
                                rect.width(), rect.left(), rect.top()
                            ));
                            vol_el.set(Some(el));
                        } else {
                            console_log!("VOL MOUNTED: FAILED!");
                        }
                    },
                    onmousedown: on_vol_down,
                    // DEBUG: fill verde neón para ver cambios
                    div {
                        class: "slider-fill debug-fill-vol",
                        style: "width: {volume:.1}%"
                    }
                    div {
                        class: "slider-knob",
                        class: if dragging_vol() { "dragging" } else { "" },
                        style: "left: {volume:.1}%"
                    }
                }

                // DEBUG: valor numérico
                span { class: "vol-debug", "{volume:.0}%" }

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