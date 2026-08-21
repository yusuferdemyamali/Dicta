mod app_state;
mod services;

use app_state::{AppState, DictationState};
use services::{feedback, settings, startup};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

#[tauri::command]
fn get_settings(state: tauri::State<AppState>) -> Result<services::settings::Settings, String> {
    Ok(state.settings.lock().unwrap().clone())
}

#[tauri::command]
fn save_settings(
    state: tauri::State<AppState>,
    app: tauri::AppHandle,
    settings: services::settings::Settings,
) -> Result<(), String> {
    // Ensure model_id defaults if empty (preserve existing files)
    let mut normalized = settings.clone();
    if normalized.model_id.trim().is_empty() {
        normalized.model_id = services::cleanup::DEFAULT_MODEL.to_string();
    }
    let mut guard = state.settings.lock().unwrap();
    *guard = normalized.clone();
    guard.save().map_err(|e| e.to_string())?;
    drop(guard);

    // Apply autostart setting without crashing
    if let Err(e) = startup::apply_autostart(&app, normalized.start_with_windows) {
        let mut err = state.last_error.lock().unwrap();
        *err = Some(format!("Autostart update failed: {}", e));
        eprintln!("autostart error: {}", e);
    } else {
        // clear autostart error if successful
        let mut err = state.last_error.lock().unwrap();
        if let Some(msg) = err.clone() {
            if msg.contains("Autostart") {
                *err = None;
            }
        }
    }
    Ok(())
}

#[tauri::command]
fn save_api_key(api_key: String) -> Result<(), String> {
    // Never log the key value
    services::credentials::save_api_key(&api_key).map_err(|e| e.to_string())?;
    println!(
        "API key saved (has_key={})",
        services::credentials::has_api_key()
    );
    Ok(())
}

#[tauri::command]
fn has_api_key() -> bool {
    services::credentials::has_api_key()
}

#[tauri::command]
fn get_api_key_status() -> bool {
    services::credentials::has_api_key()
}

#[tauri::command]
fn get_app_status(state: tauri::State<AppState>) -> Result<serde_json::Value, String> {
    let dictation = state.dictation_state.lock().unwrap().clone();
    let last_error = state.last_error.lock().unwrap().clone();
    Ok(serde_json::json!({
        "state": format!("{:?}", dictation),
        "lastError": last_error
    }))
}

fn setup_tray(app: &tauri::AppHandle) -> anyhow::Result<()> {
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings_item, &quit_item])?;

    let _tray = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => {
                if let Some(window) = app.get_webview_window("settings") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.unminimize();
                }
            }
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
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("settings") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .tooltip("Dikte - Idle")
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let initial_settings = settings::Settings::load().unwrap_or_default();

    // Ensure settings file exists with defaults (start_with_windows = true if not set)
    if let Err(e) = initial_settings.save() {
        eprintln!("failed to save initial settings: {}", e);
    }

    let app_state = AppState::new(initial_settings);

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            save_api_key,
            has_api_key,
            get_api_key_status,
            get_app_status
        ])
        .setup(move |app| {
            // Tray setup - must not crash
            if let Err(e) = setup_tray(app.handle()) {
                eprintln!("tray setup failed: {}", e);
            }

            // Autostart default enabled
            let autostart_enabled = {
                let state = app.state::<AppState>();
                let enabled = state.settings.lock().unwrap().start_with_windows;
                enabled
            };
            if let Err(e) = startup::apply_autostart(app.handle(), autostart_enabled) {
                eprintln!("autostart init failed: {}", e);
                let state = app.state::<AppState>();
                *state.last_error.lock().unwrap() = Some(format!("Autostart enable failed: {}", e));
            }

            // Global hotkey registration
            let app_handle = app.handle().clone();
            let hotkey_result = {
                let state = app.state::<AppState>();
                let res = services::hotkey::register_hotkey(&app_handle);
                let last = state.last_error.lock().unwrap().clone();
                (res, last)
            };
            match hotkey_result.0 {
                Ok(_) => {
                    println!("global hotkey registered: Ctrl+Alt+Space");
                }
                Err(e) => {
                    eprintln!("hotkey registration failed: {}", e);
                    let state = app.state::<AppState>();
                    *state.last_error.lock().unwrap() =
                        Some(format!("Hotkey Ctrl+Alt+Space unavailable: {}", e));
                }
            }

            // Initialize transcription service in background (model download/init)
            let app_handle2 = app.handle().clone();
            std::thread::spawn(move || {
                let state: tauri::State<AppState> = app_handle2.state();
                // ensure model exists and init whisper
                let local_data =
                    dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
                let app_data_dir = local_data.join("com.dikte.app");
                let models_dir = app_data_dir.join("models");
                if let Err(e) = std::fs::create_dir_all(&models_dir) {
                    eprintln!("create models dir failed: {}", e);
                    *state.last_error.lock().unwrap() =
                        Some(format!("Model directory error: {}", e));
                    return;
                }
                // Try to ensure model and init
                match services::transcription::ensure_model_and_init(&models_dir) {
                    Ok(_) => {
                        println!("whisper model ready");
                    }
                    Err(e) => {
                        eprintln!("whisper init failed: {}", e);
                        *state.last_error.lock().unwrap() =
                            Some(format!("Whisper model error: {}", e));
                        // revert to idle state if needed
                        *state.dictation_state.lock().unwrap() = DictationState::Idle;
                    }
                }
            });

            // Ensure settings window starts hidden (tray-first)
            if let Some(window) = app.get_webview_window("settings") {
                let _ = window.hide();
            }

            // Update tray tooltip for initial idle state
            feedback::update_tray_for_state(app.handle(), DictationState::Idle);

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Hide instead of closing for settings window
                if window.label() == "settings" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        });

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
