// parking_lot::Mutex — no poisoning, lock() returns guard directly.
// Remaining unwrap/expect usages are for compile-time constants or test code.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod commands;
mod log_layer;
mod state;
mod supervisor;
pub mod tray;

use std::time::Duration;

use state::AppState;
use tauri::{AppHandle, Emitter, Manager, RunEvent, WindowEvent};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_window_state::StateFlags;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

#[tauri::command]
fn get_version() -> String {
    let version = astro_up_core::version().to_string();
    tracing::debug!(command = "get_version", version, "Command completed");
    version
}

/// Check for app self-update and emit event if available.
pub(crate) async fn check_for_app_update(app: &AppHandle) {
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
            tray::set_badge_count(app, 1);
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
            tray::set_badge_count(app, 0);
        }
        Err(e) => {
            tracing::warn!("Update check failed: {e}");
        }
    }
}

/// Spawn a periodic background update check timer with panic supervision.
fn spawn_background_update_timer(app: &AppHandle) {
    let handle = app.clone();
    let state = app.state::<state::AppState>();
    let config = state.config.lock();
    let interval = if config.ui.auto_check_updates {
        config.ui.check_interval
    } else {
        Duration::from_secs(24 * 60 * 60)
    };

    supervisor::spawn_supervised(app, "update_timer", move || {
        let h = handle.clone();
        async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.tick().await;

            loop {
                ticker.tick().await;
                tracing::info!("Background update check triggered");
                check_for_app_update(&h).await;

                let count = tray::badge_count();
                if count > 0 {
                    tracing::debug!(count, "Updates available (badge already set)");
                }
            }
        }
    });
}

/// Spawn a background task that runs scheduled backups and retention pruning (#507).
fn spawn_backup_scheduler(app: &AppHandle) {
    let handle = app.clone();

    supervisor::spawn_supervised(app, "backup_scheduler", move || {
        let h = handle.clone();
        async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(3600));
            ticker.tick().await;

            loop {
                ticker.tick().await;

                let state = h.state::<state::AppState>();
                let policy = state.config.lock().backup_policy.clone();

                if !policy.scheduled_enabled {
                    continue;
                }

                tracing::debug!(schedule = %policy.schedule, "Backup scheduler tick");

                if policy.max_per_package > 0 {
                    if let Ok(reader) = state.open_catalog_reader() {
                        if let Ok(packages) = reader.list_all() {
                            for pkg in &packages {
                                if let Err(e) = state
                                    .backup_service
                                    .prune(pkg.id.as_ref(), policy.max_per_package as usize)
                                    .await
                                {
                                    tracing::warn!(package = %pkg.id, error = %e, "Failed to prune backups");
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}

/// Spawn the background scan scheduler.
///
/// On startup: if `auto_scan_on_launch` is true and no recent scan exists, trigger immediately.
/// Then, if `scan_interval` specifies a recurring duration (Hourly/Daily/Weekly), run on a timer.
fn spawn_scan_scheduler(app: &AppHandle) {
    let handle = app.clone();

    supervisor::spawn_supervised(app, "scan_scheduler", move || {
        let h = handle.clone();
        async move {
            let state = h.state::<state::AppState>();

            let (auto_scan_on_launch, scan_interval) = {
                let config = state.config.lock();
                (
                    config.ui.auto_scan_on_launch,
                    config.ui.scan_interval.clone(),
                )
            };

            let detection_store =
                astro_up_core::adapters::DetectionStore::new(state.db_path.clone());
            let last_scan_at = detection_store.last_scan_at().ok().flatten();
            let needs_initial_scan =
                last_scan_at.is_none() || is_scan_stale(&last_scan_at, &scan_interval);

            tokio::time::sleep(Duration::from_secs(5)).await;

            if auto_scan_on_launch && needs_initial_scan {
                tracing::info!(
                    last_scan = last_scan_at.as_deref().unwrap_or("never"),
                    interval = %scan_interval,
                    "Auto-scan on launch: triggering scan"
                );
                run_background_scan(&h).await;
            }

            if let Some(interval) = scan_interval.as_duration() {
                let mut ticker = tokio::time::interval(interval);
                ticker.tick().await;

                loop {
                    ticker.tick().await;
                    tracing::debug!(interval = ?scan_interval, "Periodic scan triggered");
                    run_background_scan(&h).await;
                }
            }
        }
    });
}

/// Check if the last scan is older than the configured interval.
fn is_scan_stale(
    last_scan_at: &Option<String>,
    interval: &astro_up_core::config::ScanInterval,
) -> bool {
    use astro_up_core::config::ScanInterval;

    let Some(ts) = last_scan_at else {
        return true;
    };

    let Ok(scan_time) = chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S") else {
        return true;
    };
    let scan_utc = scan_time.and_utc();
    let age = chrono::Utc::now() - scan_utc;

    match interval {
        ScanInterval::Manual => false,
        ScanInterval::OnStartup => true, // Always scan on startup
        ScanInterval::Hourly => age > chrono::Duration::hours(1),
        ScanInterval::Daily => age > chrono::Duration::days(1),
        ScanInterval::Weekly => age > chrono::Duration::weeks(1),
    }
}

/// Run a scan in the background, emitting events to the frontend.
async fn run_background_scan(handle: &AppHandle) {
    use astro_up_core::adapters::{CatalogPackageSource, DetectionStore, SqliteLedgerStore};
    use astro_up_core::detect::scanner::Scanner;
    use astro_up_core::events::Event;
    use tauri::Emitter;

    let state = handle.state::<state::AppState>();
    let catalog_path = state.catalog_manager.catalog_path().to_path_buf();
    let packages = CatalogPackageSource::new(catalog_path);
    let ledger = SqliteLedgerStore::new(state.db_path.clone());
    let scanner = Scanner::new(packages, ledger);

    let _ = handle.emit("core-event", &Event::ScanStarted);

    match scanner.scan().await {
        Ok(scan_result) => {
            let total_found = scan_result
                .results
                .iter()
                .filter(|r| r.result.is_installed())
                .count();

            // Persist detection results
            let detection_store = DetectionStore::new(state.db_path.clone());
            if let Err(e) = detection_store.save_results(&scan_result.results) {
                tracing::warn!(error = %e, "failed to persist background scan results");
            }

            let _ = handle.emit(
                "core-event",
                &Event::ScanComplete {
                    total_found: total_found as u32,
                },
            );

            tracing::info!(
                total_found,
                duration_ms = scan_result.duration.as_millis() as u64,
                "Background scan complete"
            );
        }
        Err(e) => {
            tracing::error!(error = %e, "Background scan failed");
        }
    }
}

pub fn run() {
    // Resolve data dir early for file logging
    let data_dir = directories::ProjectDirs::from("com", "nightwatch", "astro-up").map_or_else(
        || std::env::temp_dir().join("astro-up"),
        |d| d.data_dir().to_path_buf(),
    );
    let log_dir = data_dir.join("logs");
    let _ = std::fs::create_dir_all(&log_dir);

    // Load config for log settings (best-effort — defaults if DB doesn't exist yet)
    let log_config = {
        let db_path = data_dir.join("astro-up.db");
        let paths = astro_up_core::config::PathsConfig {
            data_dir,
            ..astro_up_core::config::PathsConfig::default()
        };
        astro_up_core::config::load_config(&db_path, paths, log_dir.join("astro-up.log"), &[])
            .map(|c| c.logging)
            .unwrap_or_default()
    };

    // Prune old log files before creating new ones
    astro_up_core::logging::prune_old_logs(&log_dir, log_config.max_age_days);

    // Init tracing: stderr (info) + file (debug, daily rotation) + frontend forwarding (debug+)
    let global_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug,hyper=info,reqwest=info,rustls=info"));
    let stderr_filter = EnvFilter::new("info,hyper=warn,reqwest=warn,rustls=warn");

    // File logging layer (daily rotation)
    let file_appender = tracing_appender::rolling::daily(&log_dir, "astro-up-gui.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    // Leak the guard so it lives for the entire process
    std::mem::forget(_guard);

    tracing_subscriber::registry()
        .with(global_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_filter(stderr_filter),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(non_blocking)
                .with_filter(EnvFilter::new("debug,hyper=info,reqwest=info,rustls=info")),
        )
        .with(log_layer::FrontendLogLayer)
        .init();

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
        .plugin(tauri_plugin_shell::init())
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
        .plugin(tauri_plugin_process::init())
        .manage({
            let data_dir = directories::ProjectDirs::from("com", "nightwatch", "astro-up").map_or_else(|| std::env::temp_dir().join("astro-up"), |d| d.data_dir().to_path_buf());
            std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
            AppState::new(&data_dir).expect("Failed to initialize app state")
        })
        .invoke_handler(tauri::generate_handler![
            get_version,
            commands::sync_catalog,
            commands::list_software,
            commands::get_versions,
            commands::search_catalog,
            commands::check_for_updates,
            commands::get_config,
            commands::save_config,
            commands::scan_installed,
            commands::install_software,
            commands::reinstall_software,
            commands::update_software,
            commands::update_all,
            commands::create_backup,
            commands::restore_backup,
            commands::cancel_operation,
            commands::list_backups,
            commands::backup_preview,
            commands::delete_backup,
            commands::clear_directory,
            commands::resolve_asset_selection,
            commands::get_activity,
            commands::get_last_scan,
            commands::check_survey_eligible,
            commands::dismiss_survey,
            commands::complete_survey,
        ])
        .setup(|app| {
            let start = std::time::Instant::now();

            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;

            // Wire the app handle so the tracing layer can emit to the frontend
            log_layer::set_app_handle(app.handle().clone());

            tracing::debug!("Plugins registered in {:?}", start.elapsed());

            if let Some(window) = app.get_webview_window("main") {
                // Enforce minimum size when window-state plugin restores a too-small window
                const MIN_WIDTH: u32 = 1024;
                const MIN_HEIGHT: u32 = 680;
                if let Ok(size) = window.inner_size() {
                    let w = size.width.max(MIN_WIDTH);
                    let h = size.height.max(MIN_HEIGHT);
                    if w != size.width || h != size.height {
                        let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize::new(w, h)));
                        tracing::info!(width = w, height = h, "Enforced minimum window size");
                    }
                }
                tracing::info!(label = "main", "Window created: {:?}", window.inner_size());
            }

            tray::setup(app.handle())?;
            tracing::debug!("System tray created");

            // Wire autostart to config (#652)
            #[cfg(desktop)]
            {
                use tauri_plugin_autostart::ManagerExt;
                let autostart = app.autolaunch();
                let state = app.state::<AppState>();
                let want_autostart = state.config.lock().startup.start_at_login;
                let is_enabled = autostart.is_enabled().unwrap_or(false);

                if want_autostart && !is_enabled {
                    if let Err(e) = autostart.enable() {
                        tracing::warn!("Failed to enable autostart: {e}");
                    }
                } else if !want_autostart && is_enabled {
                    if let Err(e) = autostart.disable() {
                        tracing::warn!("Failed to disable autostart: {e}");
                    }
                }
                tracing::debug!(
                    enabled = autostart.is_enabled().unwrap_or(false),
                    config = want_autostart,
                    "Autostart status"
                );
            }

            // Startup catalog sync — fetch if missing or stale
            let catalog_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = catalog_handle.state::<AppState>();
                tracing::info!("Syncing catalog...");
                let _ = catalog_handle.emit("catalog-status", "syncing");
                match state.catalog_manager.ensure_catalog().await {
                    Ok(result) => {
                        tracing::info!(result = ?result, "Catalog sync complete");
                        let _ = catalog_handle.emit("catalog-status", "ready");
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Catalog sync failed");
                        let _ = catalog_handle.emit("catalog-status", "error");
                    }
                }
            });

            // Startup self-update check (T029)
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                check_for_app_update(&handle).await;
            });

            // Background periodic update check (T030)
            spawn_background_update_timer(app.handle());

            // Background scan scheduler — auto-scan on launch and/or periodic interval
            spawn_scan_scheduler(app.handle());

            // Background backup scheduler (#507)
            spawn_backup_scheduler(app.handle());

            tracing::info!(
                version = astro_up_core::version().to_string().as_str(),
                elapsed_ms = start.elapsed().as_millis() as u64,
                "Astro-Up GUI initialized"
            );
            Ok(())
        })
        .on_window_event(|window, event| {
            fn cancel_all_and_exit(app: &AppHandle) {
                let state = app.state::<AppState>();
                let keys: Vec<String> =
                    state.operations.iter().map(|r| r.key().clone()).collect();
                for key in keys {
                    state.cancel_operation(&key);
                }
                tracing::info!("All operations cancelled, exiting");
                state
                    .quit_requested
                    .store(true, std::sync::atomic::Ordering::Relaxed);
                app.exit(0);
            }

            if let WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let state = app.state::<AppState>();

                // Read close behavior from config (FR-030, #652)
                let minimize_to_tray = state
                    .config
                    .lock()
                    .startup
                    .minimize_to_tray_on_close;

                // Prevent close — we'll handle it in all cases
                api.prevent_close();

                if state.has_active_operations() {
                    let app_clone = app.clone();
                    let window_clone = window.clone();
                    let tray = minimize_to_tray;
                    tauri::async_runtime::spawn(async move {
                        if tray {
                            // Minimize-to-tray ON: offer background or cancel
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
                                let _ = window_clone.hide();
                            } else {
                                cancel_all_and_exit(&app_clone);
                            }
                        } else {
                            // Minimize-to-tray OFF: confirm quit (no background option)
                            let answer = app_clone
                                .dialog()
                                .message("Operations are still running. Cancel them and quit?")
                                .title("Active Operations")
                                .buttons(MessageDialogButtons::OkCancelCustom(
                                    "Wait".into(),
                                    "Cancel & Quit".into(),
                                ))
                                .blocking_show();

                            if !answer {
                                cancel_all_and_exit(&app_clone);
                            }
                            // "Wait" — do nothing, window stays open
                        }
                    });
                } else if minimize_to_tray {
                    // No active operations — minimize to tray
                    let _ = window.hide();
                } else {
                    // No active operations, no tray — quit entirely
                    state
                        .quit_requested
                        .store(true, std::sync::atomic::Ordering::Relaxed);
                    app.exit(0);
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let RunEvent::ExitRequested { api, .. } = &event {
                let state = app.state::<state::AppState>();
                if !state
                    .quit_requested
                    .load(std::sync::atomic::Ordering::Relaxed)
                {
                    // Keep running in tray when all windows are closed.
                    api.prevent_exit();
                }
            }
        });
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn get_version_returns_nonempty() {
        let v = get_version();
        assert!(!v.is_empty());
    }
}
