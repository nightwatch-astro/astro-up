pub mod json;
pub mod progress;
pub mod table;

use std::io::IsTerminal;

/// Controls how output is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// TTY with colors, tables, progress bars.
    Interactive,
    /// No colors, no TUI (piped output).
    Plain,
    /// Suppress non-error output (--quiet).
    Quiet,
    /// Structured JSON to stdout.
    Json,
}

impl OutputMode {
    pub fn detect(json: bool, quiet: bool) -> Self {
        if json {
            Self::Json
        } else if quiet {
            Self::Quiet
        } else if !std::io::stdout().is_terminal() {
            Self::Plain
        } else {
            Self::Interactive
        }
    }

    /// Whether user-facing messages should be printed.
    pub const fn should_print(&self) -> bool {
        !matches!(self, Self::Quiet)
    }
}
