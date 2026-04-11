//! Lightweight task supervisor with panic recovery and sliding window restart limits.
//!
//! Critical background tasks (event forwarding, schedulers) are wrapped in a
//! supervisor that catches panics and restarts the task up to a configurable
//! number of times within a time window. When the budget is exhausted, it emits
//! a Tauri event so the frontend can alert the user.

use std::future::Future;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};

/// Default restart budget: 3 restarts within 10 minutes.
const DEFAULT_MAX_RESTARTS: u32 = 3;
const DEFAULT_WINDOW: Duration = Duration::from_secs(600);

/// Spawn a critical async task with panic recovery and restart limits.
///
/// If the task panics, it is restarted up to `max_restarts` times within
/// `window`. When the budget is exhausted, a `task-budget-exhausted` event
/// is emitted to the frontend and the task is left dead.
pub fn spawn_supervised<F, Fut>(app: &AppHandle, task_name: &'static str, factory: F)
where
    F: Fn() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let handle = app.clone();

    tauri::async_runtime::spawn(async move {
        let mut restart_count: u32 = 0;
        let mut window_start = Instant::now();

        loop {
            let task = std::panic::AssertUnwindSafe(factory());
            match futures::FutureExt::catch_unwind(task).await {
                Ok(()) => {
                    tracing::debug!(task = task_name, "supervised task completed normally");
                    break;
                }
                Err(panic_val) => {
                    let msg: &str = panic_val
                        .downcast_ref::<&str>()
                        .copied()
                        .or_else(|| panic_val.downcast_ref::<String>().map(String::as_str))
                        .unwrap_or("unknown");

                    tracing::error!(
                        task = task_name,
                        panic = msg,
                        restart_count,
                        "supervised task panicked"
                    );

                    // Reset window if expired
                    if window_start.elapsed() > DEFAULT_WINDOW {
                        restart_count = 0;
                        window_start = Instant::now();
                    }

                    restart_count += 1;

                    if restart_count > DEFAULT_MAX_RESTARTS {
                        tracing::error!(
                            task = task_name,
                            max = DEFAULT_MAX_RESTARTS,
                            "restart budget exhausted"
                        );
                        let _ = handle.emit("task-budget-exhausted", task_name);
                        break;
                    }

                    tracing::warn!(
                        task = task_name,
                        restart_count,
                        max = DEFAULT_MAX_RESTARTS,
                        "restarting"
                    );
                }
            }
        }
    });
}
