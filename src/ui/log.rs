//! Terminal status logging.
//!
//! A tiny leveled logger built on `colored`. The verbosity is set once at
//! startup via [`init_logger`] and read by the `info` / `error` / `hook` /
//! `general` helper modules.

use std::sync::atomic::{AtomicU8, Ordering};

use colored::*;

/// Verbosity of terminal output, ordered `Quiet < Normal < Verbose`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum LogLevel {
    Quiet,
    Normal,
    Verbose,
}

impl LogLevel {
    pub fn is_quiet(&self) -> bool {
        matches!(self, LogLevel::Quiet)
    }

    pub const fn as_u8(self) -> u8 {
        match self {
            LogLevel::Quiet => 0,
            LogLevel::Normal => 1,
            LogLevel::Verbose => 2,
        }
    }
}

impl From<u8> for LogLevel {
    fn from(value: u8) -> Self {
        match value {
            0 => LogLevel::Quiet,
            1 => LogLevel::Normal,
            _ => LogLevel::Verbose,
        }
    }
}

// Thread-safe mutable logger level stored as a u8 (0=Quiet, 1=Normal, 2=Verbose).
// AtomicU8 allows the level to change without rebuilding Logger.
static LEVEL: AtomicU8 = AtomicU8::new(LogLevel::Normal.as_u8());

/// Set the logger verbosity.
pub fn init_logger(level: LogLevel) {
    LEVEL.store(level.as_u8(), Ordering::SeqCst);
}

/// Read the current logger verbosity.
fn get_level() -> LogLevel {
    LEVEL.load(Ordering::SeqCst).into()
}

// Info module
pub mod info {
    use super::*;

    pub fn success(section: &str, msg: &str) {
        if get_level() >= LogLevel::Normal {
            println!(
                "{} [{}] {}",
                "✓".green().bold(),
                section.blue(),
                msg.green()
            );
        }
    }

    pub fn processed_successfully(section: &str) {
        success(section, "processed successfully");
    }
}

// Error module
pub mod error {
    use super::*;

    pub fn message(section: &str, msg: &str) {
        // Errors are always shown, regardless of the configured level.
        eprintln!("{} [{}] {}", "✗".red().bold(), section.red(), msg.red());
    }

    pub fn hook_error(section: &str, error: &str) {
        message(section, &format!("Error executing hook command: {}", error));
    }
}

// Hook module
pub mod hook {
    use super::*;

    pub fn executing(section: &str) {
        if get_level() >= LogLevel::Verbose {
            println!(
                "{} [{}] {}",
                "→".blue(),
                section.blue(),
                "Hook command executing...".blue()
            );
        }
    }

    pub fn success(section: &str) {
        if get_level() >= LogLevel::Normal {
            println!(
                "{} [{}] {}",
                "✓".green().bold(),
                section.blue(),
                "Hook command executed successfully".green()
            );
        }
    }
}

// General purpose functions
pub mod general {
    use super::*;

    pub fn info(msg: &str) {
        if get_level() >= LogLevel::Normal {
            println!("{}", msg);
        }
    }

    pub fn summary(success_count: usize, total_count: usize) {
        info(&format!(
            "{}: {} {} {} {}",
            "Summary".bold(),
            success_count.to_string().green().bold(),
            "of".white(),
            total_count.to_string().white().bold(),
            "sections processed successfully".green()
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_logger_and_levels() {
        // Initialising must not panic at any level, and subsequent calls must
        // flip the level (AtomicU8), unlike OnceLock where only the first call wins.
        init_logger(LogLevel::Quiet);
        assert_eq!(get_level(), LogLevel::Quiet);

        init_logger(LogLevel::Normal);
        assert_eq!(get_level(), LogLevel::Normal);

        init_logger(LogLevel::Verbose);
        assert_eq!(get_level(), LogLevel::Verbose);
    }
}
