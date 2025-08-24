use crate::config::Config;
use crate::error::Result;

pub mod play;
pub mod scan;
pub mod list;
pub mod search;
pub mod playlist;

pub use play::PlayCommand;
pub use scan::ScanCommand;
pub use list::ListCommand;
pub use search::SearchCommand;
pub use playlist::{PlaylistCommand, ShuffleCommand, RepeatCommand};

/// Command trait for implementing the Command pattern
/// Each CLI operation implements this trait for consistent execution
pub trait Command {
    /// Execute the command with the given configuration
    fn execute(&self, config: &Config) -> Result<()>;
    
    /// Get a description of what this command does
    #[allow(dead_code)] // Future feature: help system
    fn description(&self) -> &'static str;
}

/// Factory for creating commands based on CLI input
pub struct CommandFactory;

impl CommandFactory {
    /// Create a command from CLI arguments
    pub fn create_command(cli_command: &crate::cli::Commands) -> Box<dyn Command> {
        match cli_command {
            crate::cli::Commands::Play { .. } => Box::new(PlayCommand::new()),
            crate::cli::Commands::Scan { force, .. } => Box::new(ScanCommand::new(*force)),
            crate::cli::Commands::List { artist, album } => {
                Box::new(ListCommand::new(artist.clone(), album.clone()))
            }
            crate::cli::Commands::Search { query } => Box::new(SearchCommand::new(query.clone())),
            crate::cli::Commands::Playlist { action } => Box::new(PlaylistCommand::new(action.clone())),
            crate::cli::Commands::Shuffle { mode } => Box::new(ShuffleCommand::new(mode.clone())),
            crate::cli::Commands::Repeat { mode } => Box::new(RepeatCommand::new(mode.clone())),
        }
    }
}
