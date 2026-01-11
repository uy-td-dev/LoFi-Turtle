mod art;
mod audio;
mod cli;
mod commands;
mod config;
mod error;
mod library;
mod models;
mod services;
mod ui;

use clap::Parser;
use cli::Cli;
use commands::{Command, CommandFactory};
use config::{Config, LayoutConfig};
use error::{LofiTurtleError, Result};
use toml::Value;

/// Main entry point for the LofiTurtle music player
fn main() {
    // Initialize logging
    env_logger::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Run application with proper error handling
    if let Err(error) = run_application(cli) {
        display_error(&error);
        std::process::exit(1);
    }
}

/// Run the application with proper error handling
fn run_application(cli: Cli) -> Result<()> {
    // Create configuration from CLI arguments
    let config = Config::from_cli(&cli)?;

    // Load layout configuration
    // Check if layout_config is specified or default exists
    // If user didn't specify --layout-config, clap uses default "layout.toml"
    // We should check if that file exists, otherwise use default
    let layout_path = &cli.layout_config;

    let mut layout_config = if layout_path.exists() {
        log::info!("Loading layout config from {}", layout_path.display());
        match LayoutConfig::load_from_file(layout_path) {
            Ok(config) => config,
            Err(e) => {
                log::warn!("Failed to load layout config from {}: {}. Using defaults.", layout_path.display(), e);
                eprintln!("⚠️ Failed to load layout config from {}: {}. Using defaults.", layout_path.display(), e);
                LayoutConfig::default()
            }
        }
    } else {
        // Only log if it's not the default path, or if user explicitly asked for a file that doesn't exist
        // But since clap provides a default, we can't easily distinguish "user provided" vs "default"
        // unless we check if the path is "layout.toml" and it's missing.
        // For now, just use default silently if default file is missing.
        if layout_path.to_string_lossy() != "layout.toml" {
             log::warn!("Layout config file {} not found. Using defaults.", layout_path.display());
             eprintln!("⚠️ Layout config file {} not found. Using defaults.", layout_path.display());
        }
        LayoutConfig::default()
    };

    // Load keymap configuration if it exists
    if cli.keymap_config.exists() {
        log::info!("Loading keymap config from {}", cli.keymap_config.display());
        match std::fs::read_to_string(&cli.keymap_config) {
            Ok(content) => {
                match content.parse::<Value>() {
                    Ok(value) => {
                        // Try to find [keybindings] section, or use the whole file if it's a table
                        let keybindings = if let Some(table) = value.get("keybindings").and_then(|v| v.as_table()) {
                            Some(table)
                        } else if let Some(table) = value.as_table() {
                            Some(table)
                        } else {
                            None
                        };

                        if let Some(table) = keybindings {
                            for (key, action_val) in table {
                                if let Some(action) = action_val.as_str() {
                                    layout_config.keybindings.insert(key.clone(), action.to_string());
                                }
                            }
                        }
                    },
                    Err(e) => log::warn!("Failed to parse keymap config: {}", e),
                }
            },
            Err(e) => log::warn!("Failed to read keymap config: {}", e),
        }
    }

    // Dump layout if requested
    if let Some(dump_path) = &cli.dump_layout {
        log::info!("Dumping layout config to {}", dump_path.display());
        layout_config.save_to_file(dump_path)?;
        println!("Layout configuration dumped to {}", dump_path.display());
        return Ok(());
    }

    // Handle different commands or default to play mode
    match &cli.command {
        Some(command) => {
            // Execute the specified command
            let cmd = CommandFactory::create_command(command);
            // Pass layout config if the command supports it (PlayCommand does)
            cmd.execute_with_layout(&config, &layout_config)
        }
        None => {
            // Default behavior: check if CLI mode is requested
            if config.cli_mode {
                // Run in CLI mode - show help or basic info
                println!("🎵 LofiTurtle Music Player - CLI Mode");
                println!("Use --help to see available commands");
                println!("Run without --cli-mode for the interactive TUI interface");
                Ok(())
            } else {
                // Default behavior: start the interactive music player (TUI mode)
                let play_command = commands::PlayCommand::new();
                play_command.execute_with_layout(&config, &layout_config)
            }
        }
    }
}

/// Display user-friendly error messages
fn display_error(error: &LofiTurtleError) {
    match error {
        LofiTurtleError::DirectoryNotFound(msg) => {
            eprintln!("❌ Directory Error: {}", msg);
            eprintln!("💡 Tip: Use --music-dir to specify a different directory");
        }
        LofiTurtleError::Database(err) => {
            eprintln!("❌ Database Error: {}", err);
            eprintln!("💡 Tip: Try deleting the database file to reset");
        }
        LofiTurtleError::AudioPlayback(msg) => {
            eprintln!("❌ Audio Error: {}", msg);
            eprintln!("💡 Tip: Check if your audio drivers are working");
        }
        LofiTurtleError::UnsupportedFormat(msg) => {
            eprintln!("❌ Format Error: {}", msg);
            eprintln!("💡 Tip: Supported formats: MP3, FLAC, AAC, M4A, OGG, WAV");
        }
        LofiTurtleError::Configuration(msg) => {
            eprintln!("❌ Configuration Error: {}", msg);
            eprintln!("💡 Tip: Use --help to see available options");
        }
        LofiTurtleError::Terminal(msg) => {
            eprintln!("❌ Terminal Error: {}", msg);
            eprintln!("💡 Tip: Try running in a different terminal");
        }
        _ => {
            eprintln!("❌ Error: {}", error);
        }
    }
}
