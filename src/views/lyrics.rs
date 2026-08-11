use dioxus::prelude::*;

#[component]
pub fn Lyrics() -> Element {
    rsx! {
        div { class: "lyrics-view",
            // Info de canción actual
            div { class: "lyrics-header",
                div { class: "lyrics-cover", "🎵" }
                div { class: "lyrics-song-info",
                    div { class: "lyrics-song-title", "3 Strikes" }
                    div { class: "lyrics-song-artist", "Terror Jr" }
                }
            }

            // Letras
            div { class: "lyrics-content",
                p { class: "lyrics-line", "Three strikes and I'm out" }
                p { class: "lyrics-line", "I can't keep doing this" }
                p { class: "lyrics-line active", "Three strikes, you're out" }
                p { class: "lyrics-line", "Baby, I'm done with it" }
                p { class: "lyrics-line", "Three strikes and I'm out" }
                p { class: "lyrics-line", "I gave you all my love" }
                p { class: "lyrics-line", "But three strikes, you're out" }
                p { class: "lyrics-line", "And now I'm moving on" }
                p { class: "lyrics-line", "..." }
                p { class: "lyrics-line", "..." }
            }

            // Fuente
            div { class: "lyrics-source", "Letras proporcionadas por Genius" }
        }
    }
}