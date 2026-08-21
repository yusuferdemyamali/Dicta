use tauri_plugin_autostart::ManagerExt;

pub fn apply_autostart(app: &tauri::AppHandle, enabled: bool) -> anyhow::Result<()> {
    let autostart = app.autolaunch();
    if enabled {
        if !autostart.is_enabled().unwrap_or(false) {
            autostart.enable()?;
        }
    } else if autostart.is_enabled().unwrap_or(false) {
        autostart.disable()?;
    }
    Ok(())
}
