// use tauri::Manager;
mod audio;
use tauri_plugin_log::{Target, TargetKind};
use audio::player::AudioPlayer;
use std::sync::Mutex;
use log::{error, info};
use serde_json::Value;
// use audio::symphonia_source::SymphoniaSource;
use tauri::State;
use tauri::{
    include_image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent}, Manager,
};
// use tauri::{
//     include_image,
//     menu::{Menu, MenuItem},
//     tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
//     Listener, Manager, WindowEvent,
// };
// #[cfg(target_os = "windows")]
// use tauri::utils::config::WindowConfig;

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
fn play_file(state: State<AppState>, path: String) -> Result<Value, String> {
    let a = state.player.lock().unwrap().play_file(&path);
    match a {
        Ok(valor) => {
            info!("Operación exitosa. Datos recibidos: {:?}", valor);
            // Al no poner punto y coma aquí, este Ok es lo que la función retorna
            Ok(valor)
        }
        Err(mensaje_error) => {
            error!("Falló la operación. Motivo: {}", mensaje_error);
            // Al no poner punto y coma aquí, este Err es lo que la función retorna
            Err(mensaje_error)
        }
    }

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

//noinspection D
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let player = AudioPlayer::new().expect("No se pudo crear el reproductor");
    // std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
    //                   "--single-process \
    //      --disable-gpu \
    //      --disable-gpu-compositing \
    //      --disable-features=NetworkServiceInProcess,Translate,SharingHub,Extensions, msWebOOUI,msAutofillEdgeCoupled \
    //      --disable-crash-reporter \
    //      --disable-translate \
    //      --disable-features=InterestFeedContentSuggestions \
    //      --disable-features=CalculateNativeWinOcclusion \
    //      --disable-default-apps \
    //      --disable-sync \
    //      --js-flags='--max-semi-space-size=1 --max-old-space-size=32'"
    // );
    tauri::Builder::default()
        .setup(|app| {
            // 1. Crear un menú sencillo para el Tray (opcional, p. ej. botón Salir)
            let quit_i = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;

            // 2. Cargar el icono del Tray (asegúrate de que exista en src/icons/)
            let icon = include_image!("./icons/icon.ico");

            // 3. Construir el Tray Icon
            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        // Al hacer clic izquierdo en el tray, mostramos la ventana principal
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // 4. Intercepta el evento de la ventana para ocultarla cuando se minimice
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Resized(_) = event {
                        // Verificamos de forma segura si la ventana se encuentra minimizada
                        if let Ok(true) = window_clone.is_minimized() {
                            let _ = window_clone.hide();
                        }
                    }
                });
            }

            Ok(())
        })
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                // Define dónde quieres ver los logs (en la Terminal y/o en un archivo físico)
                .targets([
                    Target::new(TargetKind::Stdout), // Muestra logs en la consola/terminal
                    Target::new(TargetKind::LogDir { file_name: Some("logs".to_string()) }), // Opcional: Guarda logs en archivos .log
                ])
                .build(),
        )
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
