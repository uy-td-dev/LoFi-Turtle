//! Music player module
//! 
//! This module contains the core music playback logic, separated from UI concerns
//! following clean architecture principles.

pub mod playback_engine;
pub mod playlist_manager;
pub mod audio_controller;

pub use playback_engine::*;
pub use playlist_manager::*;
pub use audio_controller::*;
