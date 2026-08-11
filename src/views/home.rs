use dioxus::document::eval;
use dioxus::prelude::*;
#[component]
pub fn Home() -> Element {
    let mut path = use_signal(|| {
        String::from(
            "C:\\Users\\PC\\Downloads\\Every Breath You Take feat. Yvonne [yqNJ0S4IF_Y].webm",
        )
    });

    rsx! {
        div { class: "test-audio",
            input {
                value: "{path()}",
                oninput: move |e| path.set(e.value())
            }
            button {

                onclick: move |_| async move {
    let path = path();

    // Escapar la ruta para JSON manualmente
    let escaped_path = path
        .replace('\\', "\\\\")   // \ -> \\
        .replace('"', "\\\"")     // " -> \"
        .replace('\n', "\\n")     // newline -> \n
        .replace('\r', "\\r");    // carriage return -> \r

    // Construir JSON manual
    let json_args = format!("{{\"path\":\"{}\"}}", escaped_path);

    // Construir JS
    let js_code = format!("window.__TAURI__.core.invoke('play_file', {})", json_args);

    // Debug
    let _ = eval(&format!("console.log('JS:', {})", json_args)).await;

    // Llamar a Tauri
    let _ = eval(&js_code).await;
                },
                "▶ Reproducir"
            }
            button {
                onclick: move |_| async move {
                    let _ = eval(r#"window.__TAURI__.core.invoke('pause')"#).await;
                },
                "⏸ Pausar"
            }
            button {
                onclick: move |_| async move {
                    let _ = eval(r#"window.__TAURI__.core.invoke('resume')"#).await;
                },
                "▶ Reanudar"
            }
            button {
                onclick: move |_| async move {
                    let _ = eval(r#"window.__TAURI__.core.invoke('stop')"#).await;
                },
                "⏹ Detener"
            }
        }
    }
}
