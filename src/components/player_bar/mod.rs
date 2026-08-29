mod polling;
mod slider;

use crate::ipc::commands;
use crate::state::{use_player_state, RepeatMode};
use crate::util::RateLimitedSender;
use dioxus::prelude::*;
use dioxus::web::WebEventExt;
use polling::use_playback_polling;
use slider::{calc_pct, DragHandlers, Slider};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlElement};

#[component]
pub fn PlayerBar(show_queue: Signal<bool>) -> Element {
    let mut player = use_player_state();

    let is_playing_memo = use_memo(move || player().is_playing);
    let volume_memo = use_memo(move || player().volume);

    let mut dragging_vol = use_signal(|| false);
    let mut dragging_prog = use_signal(|| false);
    let mut vol_el: Signal<Option<HtmlElement>> = use_signal(|| None);
    let mut prog_el: Signal<Option<HtmlElement>> = use_signal(|| None);

    let should_poll = use_memo(move || is_playing_memo() && !dragging_prog());
    use_playback_polling(player, should_poll);

    let send_vol = use_hook(|| {
        RateLimitedSender::new(
            move |value: f64| {
                wasm_bindgen_futures::spawn_local(async move {
                    commands::set_volume(value / 100.0).await;
                });
            },
            60.0,
        )
    });

    // Global drag listeners: registered once and removed on unmount. Re-adding
    // them per mousedown stacks duplicate handlers and makes the bars jump.
    let drag_handlers: Rc<RefCell<Option<DragHandlers>>> = use_hook(|| Rc::new(RefCell::new(None)));

    let drag_setup = drag_handlers.clone();
    let send_vol_drag = send_vol.clone();
    use_effect(move || {
        if drag_setup.borrow().is_some() {
            return;
        }

        let win = window().expect("window");
        let doc = win.document().expect("document");

        let vol_el = vol_el;
        let prog_el = prog_el;
        let mut player = player;
        let mut dragging_vol = dragging_vol;
        let mut dragging_prog = dragging_prog;
        let send_vol_up = send_vol_drag.clone();

        let on_move = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |e: web_sys::MouseEvent| {
            let x = f64::from(e.client_x());

            if dragging_vol()
                && let Some(el) = vol_el()
            {
                let new_vol = calc_pct(&el, x) * 100.0;
                if (player.peek().volume - new_vol).abs() > 0.05 {
                    player.with_mut(|p| p.volume = new_vol);
                }
            }

            if dragging_prog()
                && let Some(el) = prog_el()
            {
                let duration = player.peek().duration;
                if duration > 0.0 {
                    let new_time = calc_pct(&el, x) * duration;
                    if (player.peek().current_time - new_time).abs() > 0.02 {
                        player.with_mut(|p| p.current_time = new_time);
                    }
                }
            }
        });

        let on_up = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |_e: web_sys::MouseEvent| {
            let was_dragging_vol = dragging_vol();
            let was_dragging_prog = dragging_prog();
            if was_dragging_vol {
                dragging_vol.set(false);
            }
            if was_dragging_prog {
                dragging_prog.set(false);
                let time = player.peek().current_time;
                wasm_bindgen_futures::spawn_local(async move {
                    commands::set_time(time).await;
                });
            }
            if was_dragging_vol {
                send_vol_up.flush(player.peek().volume);
            }
        });

        doc.add_event_listener_with_callback("mousemove", on_move.as_ref().unchecked_ref())
            .unwrap();
        doc.add_event_listener_with_callback("mouseup", on_up.as_ref().unchecked_ref())
            .unwrap();

        *drag_setup.borrow_mut() = Some(DragHandlers { on_move, on_up });
    });

    let drag_cleanup = drag_handlers;
    use_drop(move || {
        let Some(handlers) = drag_cleanup.borrow_mut().take() else {
            return;
        };
        if let Some(doc) = window().and_then(|w| w.document()) {
            let _ = doc.remove_event_listener_with_callback(
                "mousemove",
                handlers.on_move.as_ref().unchecked_ref(),
            );
            let _ = doc.remove_event_listener_with_callback(
                "mouseup",
                handlers.on_up.as_ref().unchecked_ref(),
            );
        }
    });

    use_effect(move || {
        send_vol.send(volume_memo());
    });

    let on_vol_scroll = move |e: Event<WheelData>| {
        e.prevent_default();
        let delta_y = e
            .as_web_event()
            .dyn_ref::<web_sys::WheelEvent>()
            .map_or(0.0, web_sys::WheelEvent::delta_y);

        let current = player().volume;
        let step = 5.0;
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
    };

    let on_vol_down = move |e: Event<MouseData>| {
        e.prevent_default();
        dragging_vol.set(true);
        if let Some(el) = vol_el() {
            let pct = calc_pct(&el, e.data().client_coordinates().x);
            player.with_mut(|p| p.volume = pct * 100.0);
        }
    };

    let on_prog_down = move |e: Event<MouseData>| {
        e.prevent_default();
        dragging_prog.set(true);
        if let Some(el) = prog_el() {
            let pct = calc_pct(&el, e.data().client_coordinates().x);
            let duration = player().duration;
            if duration > 0.0 {
                player.with_mut(|p| p.current_time = pct * duration);
            }
        }
    };

    let on_prog_scroll = move |e: Event<WheelData>| {
        e.prevent_default();
        let delta_y = e
            .as_web_event()
            .dyn_ref::<web_sys::WheelEvent>()
            .map_or(0.0, web_sys::WheelEvent::delta_y);

        let duration = player().duration;
        if duration <= 0.0 {
            return;
        }

        let current = player().current_time;
        let step_secs = 5.0;
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
                commands::set_time(new_time).await;
            });
        }
    };

    let on_toggle_play = move |_| {
        let is_playing_now = player().is_playing;
        let mut player = player;
        async move {
            if is_playing_now {
                commands::pause().await;
            } else {
                commands::resume().await;
            }
            player.with_mut(|p| p.is_playing = !is_playing_now);
        }
    };

    let on_prev = move |_| {
        let mut player = player;
        async move {
            player.with_mut(|p| p.go_back());
            if let Some(track) = player().current_track() {
                let path = track.path.clone();
                commands::play_file(&path).await;
            }
        }
    };

    let on_next = move |_| {
        let mut player = player;
        async move {
            player.with_mut(|p| p.advance());
            if let Some(track) = player().current_track() {
                let path = track.path.clone();
                commands::play_file(&path).await;
            }
        }
    };

    let on_repeat = move |_| player.with_mut(|p| p.toggle_repeat());
    let on_shuffle = move |_| player.with_mut(|p| p.toggle_shuffle());

    let progress_pct = use_memo(move || player().progress_pct());
    let volume_pct = use_memo(move || player().volume);
    let current_time_str = use_memo(move || player().format_time());
    let duration_str = use_memo(move || player().format_duration());

    let title = use_memo(move || {
        player()
            .metadata
            .as_ref()
            .map_or_else(|| "Sin reproduccion".to_string(), |m| m.title.clone())
    });
    let artist = use_memo(move || {
        player()
            .metadata
            .as_ref()
            .map_or_else(|| "—".to_string(), |m| m.artist.clone())
    });
    let image = use_memo(move || {
        player()
            .metadata
            .as_ref()
            .map_or_else(|| "Null".to_string(), |m| m.image.clone())
    });

    let repeat_mode = use_memo(move || player().repeat_mode);
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
                        img { src: "data:image/jpeg;base64,{image}", alt: "cover" }
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
                    Slider {
                        pct: progress_pct(),
                        dragging: is_dragging_prog,
                        class: "progress-slider",
                        on_mount: move |el| prog_el.set(Some(el)),
                        on_down: on_prog_down,
                        on_wheel: on_prog_scroll,
                    }
                    span { class: "time", "{duration_str}" }
                }

                div { class: "player-buttons",
                    button {
                        class: if is_shuffled() { "player-btn active" } else { "player-btn" },
                        onclick: on_shuffle,
                        svg { width: "20", height: "20", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round",
                            polyline { points: "16 3 21 3 21 8" }
                            line { x1: "4", y1: "20", x2: "21", y2: "3" }
                            polyline { points: "21 16 21 21 16 21" }
                            line { x1: "15", y1: "15", x2: "21", y2: "21" }
                            line { x1: "4", y1: "4", x2: "9", y2: "9" }
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

                Slider {
                    pct: volume_pct(),
                    dragging: is_dragging_vol,
                    class: "volume-slider",
                    on_mount: move |el| vol_el.set(Some(el)),
                    on_down: on_vol_down,
                    on_wheel: on_vol_scroll,
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
