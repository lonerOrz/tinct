//! Template processing engine
//!
//! This module provides traits and implementations for processing
//! templates with theme data, including built-in color filters.

mod engine;
pub(crate) mod filters;
mod processor;

pub use engine::TemplateEngineRegistry;
pub use filters::{ColorFormatType, FilterContext, FilterRegistry};
pub use processor::TemplateProcessor;
