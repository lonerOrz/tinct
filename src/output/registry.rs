//! Output format registry

use crate::core::OutputFormat;
use std::sync::Arc;

/// Registry for output format handlers
pub struct OutputRegistry {
    formats: Vec<Arc<dyn OutputFormat>>,
}

impl OutputRegistry {
    pub fn new() -> Self {
        Self {
            formats: Vec::new(),
        }
    }

    pub fn register(&mut self, format: Arc<dyn OutputFormat>) {
        self.formats.push(format);
    }

    pub fn get(&self, name: &str) -> Option<&Arc<dyn OutputFormat>> {
        self.formats.iter().find(|f| f.format_name() == name)
    }

    pub fn default_format(&self) -> Option<&Arc<dyn OutputFormat>> {
        self.formats.first()
    }
}

impl Default for OutputRegistry {
    fn default() -> Self {
        Self::new()
    }
}
