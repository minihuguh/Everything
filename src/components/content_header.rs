use crate::ipc::commands;
use dioxus::prelude::*;

#[component]
pub fn ContentHeader() -> Element {
    rsx! {
        div { class: "content-header",
            div { class: "window-controls",
                button {
                    class: "window-btn",
                    onclick: move |_| async move { commands::minimize_window().await },
                    svg { width: "12", height: "12", view_box: "0 0 12 12", fill: "currentColor",
                        rect { x: "2", y: "5.5", width: "8", height: "1" }
                    }
                }
                button {
                    class: "window-btn",
                    onclick: move |_| async move { commands::toggle_maximize().await },
                    svg { width: "12", height: "12", view_box: "0 0 12 12", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                        rect { x: "2.5", y: "2.5", width: "7", height: "7" }
                    }
                }
                button {
                    class: "window-btn close",
                    onclick: move |_| async move { commands::close_window().await },
                    svg { width: "12", height: "12", view_box: "0 0 12 12", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                        line { x1: "3", y1: "3", x2: "9", y2: "9" }
                        line { x1: "9", y1: "3", x2: "3", y2: "9" }
                    }
                }
            }
        }
    }
}