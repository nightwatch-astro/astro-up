mod commands;
mod state;
mod tray;

use state::AppState;
use tauri::{Manager, RunEvent, WindowEvent};

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
            let start = std::time::Instant::now();

            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;

            tracing::debug!("Plugins registered in {:?}", start.elapsed());

            if let Some(window) = app.get_webview_window("main") {
                tracing::debug!(label = "main", "Window created: {:?}", window.inner_size());
            }

            tray::setup(app.handle())?;
            tracing::debug!("System tray created");

            tracing::info!(
                version = astro_up_core::version().to_string().as_str(),
                elapsed_ms = start.elapsed().as_millis() as u64,
                "Astro-Up GUI initialized"
            );
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // Default: minimize to tray instead of quitting.
                // TODO: read ui.close_action from config; if "quit", don't prevent.
                // TODO: if operations active, show cancel/background prompt (T017).
                let app = window.app_handle();
                let state = app.state::<AppState>();
                if state.has_active_operations() {
                    tracing::info!("Close requested with active operations — hiding to tray");
                }
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            if let RunEvent::ExitRequested { api, .. } = &event {
                // Keep running in tray when all windows are closed.
                api.prevent_exit();
            }
        });
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
