mod commands;
mod state;
pub mod tray;

use std::time::Duration;

use state::AppState;
use tauri::{AppHandle, Emitter, Manager, RunEvent, WindowEvent};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_window_state::StateFlags;

#[tauri::command]
fn get_version() -> String {
    astro_up_core::version().to_string()
}

/// Check for app self-update and emit event if available.
async fn check_for_app_update(app: &AppHandle) {
    use tauri_plugin_updater::UpdaterExt;

    let updater = match app.updater() {
        Ok(u) => u,
        Err(e) => {
            tracing::debug!("Updater not available: {e}");
            return;
        }
    };

    match updater.check().await {
        Ok(Some(update)) => {
            tracing::info!(version = update.version.as_str(), "App update available");
            let _ = app.emit(
                "update-available",
                serde_json::json!({
                    "version": update.version,
                    "body": update.body,
                }),
            );
        }
        Ok(None) => {
            tracing::debug!("App is up to date");
        }
        Err(e) => {
            tracing::warn!("Update check failed: {e}");
        }
    }
}

/// Spawn a periodic background update check timer.
fn spawn_background_update_timer(app: &AppHandle) {
    let handle = app.clone();
    // TODO: read ui.check_interval from config (default 6h)
    let interval = Duration::from_secs(6 * 60 * 60);

    tauri::async_runtime::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.tick().await; // Skip first immediate tick (startup check already ran)

        loop {
            ticker.tick().await;
            tracing::debug!("Background update check triggered");
            check_for_app_update(&handle).await;

            // Update tray badge with available update count
            let count = tray::badge_count();
            if count > 0 {
                // If window is hidden, the update-available event already fired.
                // The tray badge is already set by the update check.
                tracing::debug!(count, "Updates available (badge already set)");
            }
        }
    });
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
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(StateFlags::all())
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_dialog::init())
        .manage({
            let data_dir = directories::ProjectDirs::from("dev", "nightwatch", "astro-up")
                .map(|d| d.data_dir().to_path_buf())
                .unwrap_or_else(|| std::env::temp_dir().join("astro-up"));
            std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
            AppState::new(&data_dir).expect("Failed to initialize app state")
        })
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

            // Wire autostart to config (T032)
            #[cfg(desktop)]
            {
                use tauri_plugin_autostart::ManagerExt;
                let autostart = app.autolaunch();
                // TODO: read ui.autostart from config; enable/disable accordingly
                // For now, leave autostart in its current state
                tracing::debug!(
                    enabled = autostart.is_enabled().unwrap_or(false),
                    "Autostart status"
                );
            }

            // Startup self-update check (T029)
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                check_for_app_update(&handle).await;
            });

            // Background periodic update check (T030)
            spawn_background_update_timer(app.handle());

            tracing::info!(
                version = astro_up_core::version().to_string().as_str(),
                elapsed_ms = start.elapsed().as_millis() as u64,
                "Astro-Up GUI initialized"
            );
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let state = app.state::<AppState>();

                // TODO: read ui.close_action from config
                // For now, default to "minimize"
                let close_action = "minimize"; // will read from config when wired

                if close_action == "quit" && !state.has_active_operations() {
                    // Quit path: let the close proceed
                    return;
                }

                // Prevent close — we'll handle it
                api.prevent_close();

                if state.has_active_operations() {
                    // T017: Prompt user when operations are active
                    let app_clone = app.clone();
                    let window_clone = window.clone();
                    tauri::async_runtime::spawn(async move {
                        let answer = app_clone
                            .dialog()
                            .message("Operations are still running. Cancel them and quit, or continue in the background?")
                            .title("Active Operations")
                            .buttons(MessageDialogButtons::OkCancelCustom(
                                "Continue in Background".into(),
                                "Cancel & Quit".into(),
                            ))
                            .blocking_show();

                        if answer {
                            // Continue in background — hide window
                            let _ = window_clone.hide();
                        } else {
                            // Cancel all operations and quit
                            let state = app_clone.state::<AppState>();
                            let keys: Vec<String> = state
                                .operations
                                .iter()
                                .map(|r| r.key().clone())
                                .collect();
                            for key in keys {
                                state.cancel_operation(&key);
                            }
                            tracing::info!("All operations cancelled, exiting");
                            app_clone.exit(0);
                        }
                    });
                } else {
                    // No active operations — minimize to tray (default behavior)
                    let _ = window.hide();
                }
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
