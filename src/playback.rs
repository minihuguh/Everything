use crate::ipc::commands;
use crate::state::{PlayerState, TrackMetadata};
use dioxus::prelude::*;

fn log(msg: &str) {
    web_sys::console::log_1(&format!("[PLAYBACK] {msg}").into());
}

pub fn reproducir_archivo_global(path: String, mut player: Signal<PlayerState>) {
    wasm_bindgen_futures::spawn_local(async move {
        log(&format!("play_file: {path}"));
        player.with_mut(|s| s.is_loading = true);

        let meta = commands::play_file(&path).await.and_then(|r| r.metadata);

        match meta {
            Some(meta) => player.with_mut(|s| {
                s.metadata = Some(TrackMetadata {
                    title: meta.title,
                    artist: meta.artist,
                    duration_secs: meta.duration,
                    path: path.clone(),
                    image: meta.image,
                });
                s.duration = meta.duration;
                s.current_time = 0.0;
                s.is_playing = true;
                s.is_loading = false;
            }),
            None => {
                log("response without metadata");
                player.with_mut(|s| s.is_loading = false);
            }
        }
    });
}
