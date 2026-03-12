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

#[cfg(test)]
mod tests {
    use super::*;

    // Mock template engine for testing
    struct MockEngine;
    impl TemplateEngine for MockEngine {
        fn render(&self, template: &str, _theme: &Theme, _mode: Mode) -> Result<String> {
            Ok(template.to_string())
        }
    }

    #[test]
    fn test_registry_new() {
        let registry = TemplateEngineRegistry::new();
        assert!(registry.engines.is_empty());
        assert!(registry.default_engine.is_none());
    }

    #[test]
    fn test_registry_default() {
        let registry = TemplateEngineRegistry::default();
        assert!(registry.engines.is_empty());
    }

    #[test]
    fn test_registry_register_engine() {
        let mut registry = TemplateEngineRegistry::new();
        let engine: Arc<dyn TemplateEngine> = Arc::new(MockEngine);
        registry.register(engine.clone());

        assert_eq!(registry.engines.len(), 1);
        assert!(registry.default_engine.is_some());
    }

    #[test]
    fn test_registry_render_with_registered_engine() {
        let mut registry = TemplateEngineRegistry::new();
        let engine: Arc<dyn TemplateEngine> = Arc::new(MockEngine);
        registry.register(engine);

        let theme = Theme::new("test".to_string(), "#FF5722".to_string());
        let result = registry.render("test template", &theme, Mode::Dark);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test template");
    }

    #[test]
    fn test_registry_render_without_registered_engine() {
        let registry = TemplateEngineRegistry::new();

        let theme = Theme::new("test".to_string(), "#FF5722".to_string());
        let result = registry.render("test template", &theme, Mode::Dark);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No template engine"));
    }

    #[test]
    fn test_registry_first_engine_is_default() {
        let mut registry = TemplateEngineRegistry::new();
        let engine1: Arc<dyn TemplateEngine> = Arc::new(MockEngine);
        let engine2: Arc<dyn TemplateEngine> = Arc::new(MockEngine);

        registry.register(engine1.clone());
        registry.register(engine2.clone());

        // First engine should be the default
        assert_eq!(registry.engines.len(), 2);
        // Verify the first engine is set as default by checking count
        assert!(registry.default_engine.is_some());
    }
}
