use dioxus::prelude::*;
use dioxus::web::WebEventExt;
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;

pub fn calc_pct(el: &HtmlElement, client_x: f64) -> f64 {
    let rect = el.get_bounding_client_rect();
    let w = rect.width();
    if w > 0.0 {
        ((client_x - rect.left()) / w).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

pub struct DragHandlers {
    pub on_move: Closure<dyn FnMut(web_sys::MouseEvent)>,
    pub on_up: Closure<dyn FnMut(web_sys::MouseEvent)>,
}

#[component]
pub fn Slider(
    pct: f64,
    dragging: bool,
    class: &'static str,
    on_mount: EventHandler<HtmlElement>,
    on_down: EventHandler<Event<MouseData>>,
    on_wheel: EventHandler<Event<WheelData>>,
) -> Element {
    let host_class = if dragging {
        format!("slider-host {class} dragging")
    } else {
        format!("slider-host {class}")
    };

    rsx! {
        div {
            class: "{host_class}",
            onmounted: move |evt: Event<MountedData>| {
                if let Ok(el) = evt.as_web_event().dyn_into::<HtmlElement>() {
                    on_mount.call(el);
                }
            },
            onmousedown: move |e: Event<MouseData>| on_down.call(e),
            onwheel: move |e: Event<WheelData>| on_wheel.call(e),
            div { class: "slider-fill", style: "width: {pct}%" }
            div {
                class: if dragging { "slider-knob dragging" } else { "slider-knob" },
                style: "left: {pct}%",
            }
        }
    }
}
