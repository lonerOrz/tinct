//! Template processing engine
//!
//! This module provides traits and implementations for processing
//! templates with theme data, including built-in color filters.

pub(crate) mod filters;
mod processor;

pub use filters::{ColorFormatType, FilterContext, FilterRegistry};
pub use processor::TemplateProcessor;
