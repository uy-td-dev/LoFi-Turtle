//! Application configuration structures
//! 
//! This module contains the main application configuration structures that were
//! previously in config.rs, now properly organized within the config module.

use crate::error::{LofiTurtleError, Result};
use crate::models::RepeatMode;
use crate::art::AlbumArtConfig;
use std::path::PathBuf;
use std::fs;
use serde::{Deserialize, Serialize};

/// Persistent settings that are saved between sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentSettings {
    pub volume: f32,
    pub shuffle: bool,
    pub repeat_mode: RepeatMode,
}

impl PersistentSettings {
    /// Get the path to the settings file
    fn settings_path() -> PathBuf {
        PathBuf::from("lofiturtle_settings.json")
    }

    /// Load persistent settings from file
    pub fn load() -> Self {
        match fs::read_to_string(Self::settings_path()) {
            Ok(content) => {
                match serde_json::from_str(&content) {
                    Ok(settings) => settings,
                    Err(_) => {
                        log::warn!("Failed to parse settings file, using defaults");
                        Self::default()
                    }
                }
            }
            Err(_) => {
                // File doesn't exist, use defaults
                Self::default()
            }
        }
    }

    /// Save persistent settings to file
    pub fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| LofiTurtleError::Configuration(format!("Failed to serialize settings: {}", e)))?;
        
        fs::write(Self::settings_path(), content)
            .map_err(|e| LofiTurtleError::Configuration(format!("Failed to save settings: {}", e)))?;
        
        Ok(())
    }

    /// Update volume and save to file
    pub fn update_volume(&mut self, volume: f32) -> Result<()> {
        self.volume = volume.clamp(0.0, 1.0);
        self.save()
    }
}

impl Default for PersistentSettings {
    fn default() -> Self {
        Self {
            volume: 0.7,
            shuffle: false,
            repeat_mode: RepeatMode::None,
        }
    }
}

/// Configuration for the LofiTurtle music player
#[derive(Debug, Clone)]
pub struct Config {
    pub music_dir: PathBuf,
    pub database_path: PathBuf,
    pub verbose: bool,
    pub no_scan: bool,
    pub tick_rate_ms: u64,
    pub default_volume: f32,
    pub show_art: bool,
    pub shuffle: bool,
    pub repeat_mode: RepeatMode,
    pub album_art_config: AlbumArtConfig,
    pub cli_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            music_dir: crate::cli::Cli::default_music_dir(),
            database_path: PathBuf::from("music_library.db"),
            verbose: false,
            no_scan: false,
            tick_rate_ms: 250,
            default_volume: 0.7,
            show_art: true,
            shuffle: false,
            repeat_mode: RepeatMode::None,
            album_art_config: AlbumArtConfig::default(),
            cli_mode: false,
        }
    }
}

/// Builder pattern implementation for Config
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    music_dir: Option<PathBuf>,
    database_path: Option<PathBuf>,
    verbose: Option<bool>,
    no_scan: Option<bool>,
    tick_rate_ms: Option<u64>,
    default_volume: Option<f32>,
    show_art: Option<bool>,
    shuffle: Option<bool>,
    repeat_mode: Option<RepeatMode>,
    album_art_config: Option<AlbumArtConfig>,
    cli_mode: Option<bool>,
}

impl ConfigBuilder {
    /// Create a new ConfigBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the music directory
    pub fn music_dir<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.music_dir = Some(path.into());
        self
    }

    /// Set the database path
    pub fn database_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.database_path = Some(path.into());
        self
    }

    /// Enable verbose logging
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = Some(verbose);
        self
    }

    /// Disable library scanning on startup
    pub fn no_scan(mut self, no_scan: bool) -> Self {
        self.no_scan = Some(no_scan);
        self
    }

    /// Set the tick rate in milliseconds
    #[allow(dead_code)] // Future feature: configurable tick rate
    pub fn tick_rate_ms(mut self, ms: u64) -> Self {
        self.tick_rate_ms = Some(ms);
        self
    }

    /// Set the default volume (0.0 to 1.0)
    #[allow(dead_code)] // Future feature: configurable volume
    pub fn default_volume(mut self, volume: f32) -> Self {
        if volume < 0.0 || volume > 1.0 {
            log::warn!("Volume should be between 0.0 and 1.0, got {}", volume);
        }
        self.default_volume = Some(volume.clamp(0.0, 1.0));
        self
    }

    /// Enable or disable album art display
    pub fn show_art(mut self, show_art: bool) -> Self {
        self.show_art = Some(show_art);
        self
    }

    /// Enable or disable shuffle mode
    pub fn shuffle(mut self, shuffle: bool) -> Self {
        self.shuffle = Some(shuffle);
        self
    }

    /// Set the repeat mode
    pub fn repeat_mode(mut self, mode: RepeatMode) -> Self {
        self.repeat_mode = Some(mode);
        self
    }

    /// Set the album art configuration
    pub fn album_art_config(mut self, config: AlbumArtConfig) -> Self {
        self.album_art_config = Some(config);
        self
    }
    
    /// Enable or disable CLI mode
    pub fn cli_mode(mut self, cli_mode: bool) -> Self {
        self.cli_mode = Some(cli_mode);
        self
    }

    /// Build the configuration, validating all settings
    pub fn build(self) -> Result<Config> {
        let default_config = Config::default();
        
        let music_dir = self.music_dir.unwrap_or(default_config.music_dir);
        let database_path = self.database_path.unwrap_or(default_config.database_path);
        
        // Validate music directory exists
        if !music_dir.exists() {
            return Err(LofiTurtleError::DirectoryNotFound(
                format!("Music directory '{}' does not exist", music_dir.display())
            ));
        }

        if !music_dir.is_dir() {
            return Err(LofiTurtleError::Configuration(
                format!("'{}' is not a directory", music_dir.display())
            ));
        }

        // Validate tick rate
        let tick_rate_ms = self.tick_rate_ms.unwrap_or(default_config.tick_rate_ms);
        if tick_rate_ms == 0 {
            return Err(LofiTurtleError::Configuration(
                "Tick rate must be greater than 0".to_string()
            ));
        }

        Ok(Config {
            music_dir,
            database_path,
            verbose: self.verbose.unwrap_or(default_config.verbose),
            no_scan: self.no_scan.unwrap_or(default_config.no_scan),
            tick_rate_ms,
            default_volume: self.default_volume.unwrap_or(default_config.default_volume),
            show_art: self.show_art.unwrap_or(default_config.show_art),
            shuffle: self.shuffle.unwrap_or(default_config.shuffle),
            repeat_mode: self.repeat_mode.unwrap_or(default_config.repeat_mode),
            album_art_config: self.album_art_config.unwrap_or(default_config.album_art_config),
            cli_mode: self.cli_mode.unwrap_or(default_config.cli_mode),
        })
    }
}

impl Config {
    /// Create a new ConfigBuilder
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    /// Create configuration from CLI arguments
    pub fn from_cli(cli: &crate::cli::Cli) -> Result<Self> {
        let music_dir = cli.validate_music_dir()?;
        
        // Convert CLI repeat mode to internal repeat mode
        let repeat_mode = match &cli.repeat {
            Some(crate::cli::RepeatModeArg::None) => RepeatMode::None,
            Some(crate::cli::RepeatModeArg::Single) => RepeatMode::Single,
            Some(crate::cli::RepeatModeArg::Playlist) => RepeatMode::Playlist,
            None => RepeatMode::None,
        };
        
        // Determine show_art: default true, but can be disabled with --no-art
        let show_art = !cli.no_art; // Default true unless --no-art is specified
        
        // Create album art configuration
        let album_art_config = AlbumArtConfig::builder()
            .show_art(show_art)
            .build();
        
        Self::builder()
            .music_dir(music_dir)
            .database_path(&cli.database)
            .verbose(cli.verbose)
            .no_scan(cli.no_scan)
            .show_art(show_art)
            .shuffle(cli.shuffle)
            .repeat_mode(repeat_mode)
            .album_art_config(album_art_config)
            .cli_mode(cli.cli_mode)
            .build()
    }
}
