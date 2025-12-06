use colored::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Quiet,
    Normal,
    Verbose,
}

pub struct Logger {
    level: LogLevel,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Self { level }
    }

    pub fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    pub fn log(&self, level: LogLevel, message: &str) {
        if self.level as u8 >= level as u8 {
            match level {
                LogLevel::Quiet => println!("{}", message),
                LogLevel::Normal => println!("{}", message),
                LogLevel::Verbose => eprintln!("{}", message),
            }
        }
    }

    pub fn info(&self, message: &str) {
        if self.level as u8 >= LogLevel::Normal as u8 {
            println!("{}", message);
        }
    }

    pub fn verbose(&self, message: &str) {
        if self.level as u8 >= LogLevel::Verbose as u8 {
            eprintln!("{}", message);
        }
    }

    pub fn success(&self, section: &str, message: &str) {
        if self.level as u8 >= LogLevel::Normal as u8 {
            println!("{} [{}] {}", "✓".green().bold(), section.blue(), message.green());
        }
    }

    pub fn error(&self, section: &str, message: &str) {
        if self.level as u8 >= LogLevel::Quiet as u8 {  // Always show errors
            eprintln!("{} [{}] {}", "✗".red().bold(), section.red(), message.red());
        }
    }

    pub fn hook_executing(&self, section: &str) {
        if self.level as u8 >= LogLevel::Verbose as u8 {
            println!("[{}] {} Hook command executing...", section.blue(), "✓".green());
        }
    }

    pub fn hook_success(&self, section: &str) {
        if self.level as u8 >= LogLevel::Normal as u8 {
            println!("[{}] {} Hook command executed successfully", section.blue(), "✓".green());
        }
    }

    pub fn hook_error(&self, section: &str, error: &str) {
        eprintln!("[{}] {} Error executing hook command: {}", section.red(), "✗".red(), error.red());
    }

    pub fn is_verbose(&self) -> bool {
        self.level == LogLevel::Verbose
    }
}

// Global logger instance
static mut LOGGER: Option<Logger> = None;
static mut LOGGER_INIT: std::sync::Once = std::sync::Once::new();

pub fn init_logger(level: LogLevel) {
    unsafe {
        LOGGER_INIT.call_once(|| {
            LOGGER = Some(Logger::new(level));
        });
    }
}

pub fn log(level: LogLevel, message: &str) {
    unsafe {
        if let Some(ref logger) = LOGGER {
            logger.log(level, message);
        }
    }
}

pub fn info(message: &str) {
    unsafe {
        if let Some(ref logger) = LOGGER {
            logger.info(message);
        }
    }
}

pub fn verbose(message: &str) {
    unsafe {
        if let Some(ref logger) = LOGGER {
            logger.verbose(message);
        }
    }
}

pub fn success(section: &str, message: &str) {
    unsafe {
        if let Some(ref logger) = LOGGER {
            logger.success(section, message);
        }
    }
}

pub fn error(section: &str, message: &str) {
    unsafe {
        if let Some(ref logger) = LOGGER {
            logger.error(section, message);
        }
    }
}

pub fn hook_executing(section: &str) {
    unsafe {
        if let Some(ref logger) = LOGGER {
            logger.hook_executing(section);
        }
    }
}

pub fn hook_success(section: &str) {
    unsafe {
        if let Some(ref logger) = LOGGER {
            logger.hook_success(section);
        }
    }
}

pub fn hook_error(section: &str, error: &str) {
    unsafe {
        if let Some(ref logger) = LOGGER {
            logger.hook_error(section, error);
        }
    }
}

pub fn is_verbose() -> bool {
    unsafe {
        if let Some(ref logger) = LOGGER {
            logger.is_verbose()
        } else {
            false
        }
    }
}