// LoFi Turtle Library Interface

// Core modules
pub mod art;
pub mod audio;
pub mod cli;
pub mod commands;
pub mod config;
pub mod error;
pub mod library;
pub mod models;
pub mod services;
pub mod ui;

// Re-export commonly used types for convenience
pub use error::{LofiTurtleError, Result};
pub use ui::{ThemeManager, Themes};

// Re-export new config system
pub use config::{LayoutConfig};
