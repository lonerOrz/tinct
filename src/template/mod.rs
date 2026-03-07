//! Template processing engine
//!
//! This module provides traits and implementations for processing
//! templates with theme data.

mod engine;
mod filters;
mod processor;

pub use engine::TemplateEngineRegistry;
pub use filters::Filter;
pub use processor::TemplateProcessor;
