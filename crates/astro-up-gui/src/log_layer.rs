//! Tracing layer that emits log entries to the frontend via Tauri events.

use std::fmt;
use std::sync::{Arc, OnceLock};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tracing::Subscriber;
use tracing::field::{Field, Visit};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;

/// Global app handle — set once in setup, read by the tracing layer.
static APP_HANDLE: OnceLock<Arc<AppHandle>> = OnceLock::new();

/// Store the app handle so the tracing layer can emit to the frontend.
pub fn set_app_handle(app: AppHandle) {
    let _ = APP_HANDLE.set(Arc::new(app));
}

#[derive(Clone, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    target: String,
    message: String,
}

/// Tracing layer that forwards log events to the Tauri frontend.
pub struct FrontendLogLayer;

struct MessageVisitor {
    message: String,
    fields: Vec<String>,
}

impl MessageVisitor {
    const fn new() -> Self {
        Self {
            message: String::new(),
            fields: Vec::new(),
        }
    }

    fn result(self) -> String {
        if self.fields.is_empty() {
            self.message
        } else if self.message.is_empty() {
            self.fields.join(", ")
        } else {
            format!("{} ({})", self.message, self.fields.join(", "))
        }
    }
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        } else {
            self.fields.push(format!("{}={:?}", field.name(), value));
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else {
            self.fields.push(format!("{}={}", field.name(), value));
        }
    }
}

impl<S> Layer<S> for FrontendLogLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let Some(app) = APP_HANDLE.get() else {
            return; // App handle not yet set
        };

        let metadata = event.metadata();
        let level = metadata.level();

        // Skip trace to avoid flooding
        if *level == tracing::Level::TRACE {
            return;
        }

        let mut visitor = MessageVisitor::new();
        event.record(&mut visitor);

        let entry = LogEntry {
            timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            level: level.to_string().to_lowercase(),
            target: metadata.target().to_string(),
            message: visitor.result(),
        };

        let _ = app.emit("backend-log", &entry);
    }
}
