pub mod json;
pub mod progress;
pub mod table;

use std::io::IsTerminal;

/// Controls how output is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// TTY with colors, tables, ratatui TUI.
    Interactive,
    /// No colors, no TUI (piped output or --quiet).
    Plain,
    /// Structured JSON to stdout.
    Json,
}

impl OutputMode {
    pub fn detect(json: bool, quiet: bool) -> Self {
        if json {
            Self::Json
        } else if !std::io::stdout().is_terminal() || quiet {
            Self::Plain
        } else {
            Self::Interactive
        }
    }
}
