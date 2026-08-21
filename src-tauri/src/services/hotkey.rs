use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

pub fn register_hotkey(app: &tauri::AppHandle) -> anyhow::Result<()> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::Space);

    let app_handle = app.clone();
    app.global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                crate::app_state::AppState::handle_hotkey(&app_handle);
            }
        })?;

    Ok(())
}
