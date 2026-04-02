mod commands;
mod state;
mod tray;

use state::AppState;
use tauri::Manager;

#[tauri::command]
fn get_version() -> String {
    astro_up_core::version().to_string()
}

pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }));
    }

    builder
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            get_version,
            commands::list_software,
            commands::search_catalog,
            commands::check_for_updates,
            commands::get_config,
            commands::save_config,
            commands::scan_installed,
            commands::install_software,
            commands::update_software,
            commands::create_backup,
            commands::restore_backup,
            commands::cancel_operation,
        ])
        .setup(|app| {
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;

            tracing::info!("Astro-Up GUI initialized");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_version_returns_nonempty() {
        let v = get_version();
        assert!(!v.is_empty());
    }
}
