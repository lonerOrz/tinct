//! Terminal status logging.
//!
//! A tiny leveled logger built on `colored`. The verbosity is set once at
//! startup via [`init_logger`] and read by the `info` / `error` / `hook` /
//! `general` helper modules.

use std::sync::OnceLock;

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
}

// Global logger instance using thread-safe OnceLock
static LOGGER: OnceLock<Logger> = OnceLock::new();

pub struct Logger {
    level: LogLevel,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Self { level }
    }
}

pub fn init_logger(level: LogLevel) {
    LOGGER.get_or_init(|| Logger::new(level));
}

// Info module
pub mod info {
    use super::*;

    pub fn success(section: &str, msg: &str) {
        if let Some(logger) = LOGGER.get()
            && logger.level >= LogLevel::Normal
        {
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
        if let Some(logger) = LOGGER.get()
            && logger.level >= LogLevel::Verbose
        {
            println!(
                "{} [{}] {}",
                "→".blue(),
                section.blue(),
                "Hook command executing...".blue()
            );
        }
    }

    pub fn success(section: &str) {
        if let Some(logger) = LOGGER.get()
            && logger.level >= LogLevel::Normal
        {
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
        if let Some(logger) = LOGGER.get()
            && logger.level >= LogLevel::Normal
        {
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
    fn test_logger_creation() {
        let logger = Logger::new(LogLevel::Normal);
        assert_eq!(logger.level, LogLevel::Normal);
    }

    #[test]
    fn test_logger_verbose_level() {
        let logger = Logger::new(LogLevel::Verbose);
        assert_eq!(logger.level, LogLevel::Verbose);
    }

    #[test]
    fn test_log_level_ordering() {
        // Verify all log levels exist and are distinct
        assert_ne!(LogLevel::Quiet, LogLevel::Normal);
        assert_ne!(LogLevel::Normal, LogLevel::Verbose);
        assert_ne!(LogLevel::Quiet, LogLevel::Verbose);
    }

    #[test]
    fn test_init_logger() {
        // Should not panic when initializing logger
        init_logger(LogLevel::Quiet);
        init_logger(LogLevel::Normal);
        init_logger(LogLevel::Verbose);
    }

    #[test]
    fn test_logger_traits() {
        // Verify Debug, Clone, Copy, PartialEq traits work
        let level1 = LogLevel::Quiet;
        let level2 = level1;
        assert_eq!(level1, level2);

        let cloned = level1;
        assert_eq!(level1, cloned);
    }
}
