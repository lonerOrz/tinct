//! Template engine registry for managing multiple template engines

use crate::core::{Mode, Result, TemplateEngine, Theme};
use std::sync::Arc;

/// A registry that manages multiple template engines
pub struct TemplateEngineRegistry {
    engines: Vec<Arc<dyn TemplateEngine>>,
    default_engine: Option<Arc<dyn TemplateEngine>>,
}

impl TemplateEngineRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            engines: Vec::new(),
            default_engine: None,
        }
    }

    /// Register a template engine
    pub fn register(&mut self, engine: Arc<dyn TemplateEngine>) {
        if self.default_engine.is_none() {
            self.default_engine = Some(engine.clone());
        }
        self.engines.push(engine);
    }

    /// Render a template using the default engine
    pub fn render(&self, template: &str, theme: &Theme, mode: Mode) -> Result<String> {
        if let Some(engine) = &self.default_engine {
            return engine.render(template, theme, mode);
        }

        Err(crate::core::Error::Template(
            "No template engine registered".to_string(),
        ))
    }
}

impl Default for TemplateEngineRegistry {
    fn default() -> Self {
        Self::new()
    }
}
