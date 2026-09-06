mod audio;
mod state;
mod commands;
mod utils;

use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_dialog::{DialogExt, FilePath};
use audio::player::AudioPlayer;
use std::sync::{Mutex};
use log::{debug, error, info, warn};
// use audio::symphonia_source::SymphoniaSource;

use tauri::Manager;
use crate::commands::*;

pub struct AppState {
    player: Mutex<AudioPlayer>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

//noinspection D
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let player = AudioPlayer::new().expect("No se pudo crear el reproductor");
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: Some("logs".to_string()) }),
                ])
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            player: Mutex::new(player),
        })
        .invoke_handler(tauri::generate_handler![
            minimize_window,
            toggle_maximize,
            close_window,
            play_file,
            pause,
            resume,
            stop,
            set_volume,
            is_playing,
            get_time,
            set_time,
            select_document,
            get_metadata,
            open_playlist
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
