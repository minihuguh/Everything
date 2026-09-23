use crate::state::TrackMetadata;
use crate::AppState;
use log::{debug, error, info};
use serde_json::Value;
use std::fs;
use std::sync::mpsc;
use std::time::Duration;
use tauri::State;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_fs::FilePath;
use parser::{parse, Playlist};

#[tauri::command]
pub fn minimize_window(window: tauri::WebviewWindow) {
    let _ = window.minimize();
}

#[tauri::command]
pub fn toggle_maximize(window: tauri::WebviewWindow) {
    if window.is_maximized().unwrap_or(false) {
        let _ = window.unmaximize();
    } else {
        let _ = window.maximize();
    }
}

#[tauri::command]
pub fn close_window(window: tauri::WebviewWindow) {
    let _ = window.close();
}

#[tauri::command]
pub fn play_file(state: State<AppState>, path: String) -> Result<Value, String> {
    info!("Operación exitosa. Datos a enviar: {:?}", path);

    let a = state.player.lock().unwrap().play_file(&path);
    match a {
        Ok(valor) => {
            // info!("Operación exitosa. Datos recibidos: {:?}", valor);
            Ok(valor)
        }
        Err(mensaje_error) => {
            error!("Falló la operación. Motivo: {}", mensaje_error);
            Err(mensaje_error)
        }
    }
}

#[tauri::command]
pub fn pause(state: State<AppState>) {
    state.player.lock().unwrap().pause();
}

#[tauri::command]
pub fn resume(state: State<AppState>) {
    state.player.lock().unwrap().resume();
}

#[tauri::command]
pub fn stop(state: State<AppState>) {
    state.player.lock().unwrap().stop();
}

#[tauri::command]
pub fn set_volume(state: State<AppState>, volume: f32) {
    state.player.lock().unwrap().set_volume(volume);
}

#[tauri::command]
pub fn get_time(state: State<AppState>) -> u64 {
    let valor = state.player.lock().unwrap().get_time();
    info!("Operación exitosa. Datos recibidos: {:?}", valor.to_string());
    valor
}

#[tauri::command]
pub fn set_time(state: State<AppState>, time_in_seconds: u64) -> Result<(), String> {
    let duration = Duration::from_secs(time_in_seconds);
    // let temp_duration = Duration::from_secs(30);
    let temp = state.player.lock().unwrap().set_time(duration);
    info!("Datos recibidos desde set_time: {:?}", temp);

    Ok(())
}

#[tauri::command]
pub fn open_playlist<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> Result<Playlist, String> {
    let (tx, rx) = mpsc::channel();

    app.dialog()
        .file()
        .add_filter("Text Files", &["xspf", "xml"])
        .pick_file(move |file_path| {
            let _ = tx.send(file_path);
        });

    // let response = rx.recv().map_err(|e| e.to_string())?;

    let file = rx.recv().unwrap_or(None);
    let xml = fs::read_to_string(file.unwrap().to_string()).unwrap();
    let playlist = parse(&xml);
    // info!("Respuesta enviada desde Tauri: {:?}", serde_json::to_string(&playlist));
    // debug!("Datos recibidos desde PARSER: {:?}", temp);
    playlist
}

#[tauri::command]
pub fn select_document<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> Option<FilePath> {
    let (tx, rx) = mpsc::channel();

    app.dialog()
        .file()
        .add_filter("Text Files", &["opus", "md"])
        .pick_file(move |file_path| {
            let _ = tx.send(file_path);
        });

    // let response = rx.recv().map_err(|e| e.to_string())?;

    rx.recv().unwrap_or(None)
}

#[tauri::command]
pub fn get_metadata(state: State<AppState>, path: String) -> TrackMetadata {
    info!("Operación exitosa. Datos a enviar: {:?}", path);

    let a = state.player.lock().unwrap().extract_metadata(&path);
    match a {
        Ok(valor) => {
            // info!("Operación exitosa. Datos recibidos: {:?}", valor);
            valor
        }
        Err(mensaje_error) => {
            error!("Falló la operación. Motivo: {}", mensaje_error);
            TrackMetadata {
                title: "".to_string(),
                artist: "".to_string(),
                duration: 0.0,
                path,
                image: "".to_string(),
            }
        }
    }
}

#[tauri::command]
pub fn is_playing(state: State<AppState>) -> bool {
    state.player.lock().unwrap().is_playing()
}