pub mod color;
pub mod config;
pub mod log;
pub mod preview;

// Module re-exports for backward compatibility
pub use color::*;
pub use config::*;
pub use log::*;

// New modular architecture
pub mod filter;
pub mod output_handler;
pub mod palette_generator;
pub mod template_processor;
pub mod terminal_sender;
pub mod theme_loader;

pub use filter::*;
pub use output_handler::*;
pub use palette_generator::*;
pub use template_processor::*;
pub use terminal_sender::*;
pub use theme_loader::*;
