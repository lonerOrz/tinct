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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_output_new() {
        let output = TerminalOutput::new();
        assert_eq!(output.format_name(), "terminal");
    }

    #[test]
    fn test_terminal_output_default() {
        let output = TerminalOutput::default();
        assert_eq!(output.format_name(), "terminal");
    }

    #[test]
    fn test_terminal_output_format_name() {
        let output = TerminalOutput::new();
        assert_eq!(output.format_name(), "terminal");
    }

    #[test]
    fn test_terminal_output_write() {
        let output = TerminalOutput::new();
        // Note: We can't easily test the actual print output,
        // but we can verify it doesn't error
        let result = output.write("test content", "");
        assert!(result.is_ok());
    }
}
