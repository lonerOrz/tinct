//! Terminal color sequence output

use crate::core::{OutputFormat, Result};

/// Output to terminal using ANSI escape sequences
pub struct TerminalOutput;

impl TerminalOutput {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TerminalOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormat for TerminalOutput {
    fn write(&self, content: &str, _destination: &str) -> Result<()> {
        // For terminal output, we just print to stdout
        print!("{}", content);
        Ok(())
    }

    fn format_name(&self) -> &str {
        "terminal"
    }
}
