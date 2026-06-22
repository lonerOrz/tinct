use colored::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Quiet,
    Normal,
    Verbose,
}

use std::sync::OnceLock;

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

#[allow(dead_code)]
pub fn is_verbose() -> bool {
    if let Some(logger) = LOGGER.get() {
        logger.level == LogLevel::Verbose
    } else {
        false
    }
}

// Info module
pub mod info {
    use super::*;

    #[allow(dead_code)]
    pub fn message(section: &str, msg: &str) {
        if let Some(logger) = LOGGER.get() {
            if logger.level as u8 >= LogLevel::Normal as u8 {
                println!("{} [{}] {}", "ℹ".blue(), section.blue(), msg.blue());
            }
        }
    }

    pub fn success(section: &str, msg: &str) {
        if let Some(logger) = LOGGER.get() {
            if logger.level as u8 >= LogLevel::Normal as u8 {
                println!(
                    "{} [{}] {}",
                    "✓".green().bold(),
                    section.blue(),
                    msg.green()
                );
            }
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
        if let Some(logger) = LOGGER.get() {
            if logger.level as u8 >= LogLevel::Quiet as u8 {
                // Always show errors
                eprintln!("{} [{}] {}", "✗".red().bold(), section.red(), msg.red());
            }
        }
    }

    pub fn hook_error(section: &str, error: &str) {
        message(section, &format!("Error executing hook command: {}", error));
    }
}

// Hook module
pub mod hook {
    use super::*;

    pub fn executing(section: &str) {
        if let Some(logger) = LOGGER.get() {
            if logger.level as u8 >= LogLevel::Verbose as u8 {
                println!(
                    "{} [{}] {}",
                    "→".blue(),
                    section.blue(),
                    "Hook command executing...".blue()
                );
            }
        }
    }

    pub fn success(section: &str) {
        if let Some(logger) = LOGGER.get() {
            if logger.level as u8 >= LogLevel::Normal as u8 {
                println!(
                    "{} [{}] {}",
                    "✓".green().bold(),
                    section.blue(),
                    "Hook command executed successfully".green()
                );
            }
        }
    }
}

// General purpose functions
pub mod general {
    use super::*;
    use colored::Colorize;

    pub fn info(msg: &str) {
        if let Some(logger) = LOGGER.get() {
            if logger.level as u8 >= LogLevel::Normal as u8 {
                println!("{}", msg);
            }
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
    fn test_is_verbose_default() {
        // Without initialization, is_verbose should return false
        assert!(!is_verbose());
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
