//! Terminal user interface: logging and color preview.
//!
//! Everything that renders to the user's terminal — status logging and the
//! MD3 color preview — lives here, keeping presentation concerns out of the
//! domain modules.

pub mod log;
pub mod preview;
