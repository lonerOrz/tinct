//! Theme loader registry for managing multiple theme loaders

use crate::core::{Result, Theme, ThemeLoader};
use std::sync::Arc;

/// A registry that manages multiple theme loaders
pub struct ThemeLoaderRegistry {
    loaders: Vec<Arc<dyn ThemeLoader>>,
}

impl ThemeLoaderRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            loaders: Vec::new(),
        }
    }

    /// Register a new theme loader
    pub fn register(&mut self, loader: Arc<dyn ThemeLoader>) {
        self.loaders.push(loader);
    }

    /// Load a theme from the given source
    ///
    /// This will try each registered loader in order until one succeeds
    pub fn load(&self, source: &str) -> Result<Theme> {
        for loader in &self.loaders {
            if loader.can_load(source) {
                return loader.load(source);
            }
        }
        Err(crate::core::Error::Theme(format!(
            "No suitable loader found for source: {}",
            source
        )))
    }
}

impl Default for ThemeLoaderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
