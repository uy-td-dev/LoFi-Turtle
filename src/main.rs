// Clean Architecture modules
mod domain;
mod application;
mod infrastructure;
mod shared;

// Legacy modules (to be refactored)
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
use config::Config;
use error::{LofiTurtleError, Result};

/// Main entry point for the LofiTurtle music player
#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Run application with proper error handling
    if let Err(error) = run_application(cli).await {
        display_error(&error);
        std::process::exit(1);
    }
}

/// Run the application with proper error handling
async fn run_application(cli: Cli) -> Result<()> {
    // Create configuration from CLI arguments
    let config = Config::from_cli(&cli)?;

    // Handle different commands or default to play mode
    match &cli.command {
        Some(command) => {
            // Execute the specified command
            let cmd = CommandFactory::create_command(command);
            cmd.execute(&config).await
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
                play_command.execute(&config).await
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
