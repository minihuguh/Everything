mod audio;

use std::sync::Mutex;
use audio::player::AudioPlayer;
use audio::symphonia_source::SymphoniaSource;
use tauri::State;

pub struct AppState {
    player: Mutex<AudioPlayer>,
}


// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn minimize_window(window: tauri::WebviewWindow) {
    let _ = window.minimize();
}

#[tauri::command]
fn toggle_maximize(window: tauri::WebviewWindow) {
    if window.is_maximized().unwrap_or(false) {
        let _ = window.unmaximize();
    } else {
        let _ = window.maximize();
    }
}

#[tauri::command]
fn close_window(window: tauri::WebviewWindow) {
    let _ = window.close();
}

#[tauri::command]
fn play_file(state: State<AppState>, path: String) -> Result<(), String> {
    state.player.lock().unwrap().play_file(&path)
}

#[tauri::command]
fn pause(state: State<AppState>) {
    state.player.lock().unwrap().pause();
}

#[tauri::command]
fn resume(state: State<AppState>) {
    state.player.lock().unwrap().resume();
}

#[tauri::command]
fn stop(state: State<AppState>) {
    state.player.lock().unwrap().stop();
}

#[tauri::command]
fn set_volume(state: State<AppState>, volume: f32) {
    state.player.lock().unwrap().set_volume(volume);
}

#[tauri::command]
fn is_playing(state: State<AppState>) -> bool {
    state.player.lock().unwrap().is_playing()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let player = AudioPlayer::new().expect("No se pudo crear el reproductor");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
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
            is_playing
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

