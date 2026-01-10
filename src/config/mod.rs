//! Configuration management module
//! 
//! This module handles loading, parsing, and merging of layout configurations.
//! It provides a clean interface for managing default and user-defined settings.

pub mod layout_config;
pub mod defaults;
pub mod app_config;

pub use layout_config::LayoutConfig;
pub use app_config::{Config, PersistentSettings};
