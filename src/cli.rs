use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Repeat mode argument for CLI
#[derive(Debug, Clone, ValueEnum)]
pub enum RepeatModeArg {
    None,
    Single,
    Playlist,
}

/// LofiTurtle - A terminal-based music player
#[derive(Parser, Debug)]
#[command(name = "lofiturtle")]
#[command(about = "A beautiful terminal-based music player written in Rust")]
#[command(version)]
pub struct Cli {
    /// Music directory to scan and play from
    #[arg(short, long, value_name = "DIR")]
    pub music_dir: Option<PathBuf>,

    /// Database file path
    #[arg(short, long, value_name = "FILE", default_value = "music_library.db")]
    pub database: PathBuf,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Disable library scanning on startup
    #[arg(long)]
    pub no_scan: bool,

    /// Show album art in terminal (enabled by default)
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub show_art: bool,

    /// Disable album art in terminal
    #[arg(long, conflicts_with = "show_art")]
    pub no_art: bool,


    /// Enable shuffle mode
    #[arg(long)]
    pub shuffle: bool,

    /// Set repeat mode (none, single, playlist)
    #[arg(long, value_enum)]
    pub repeat: Option<RepeatModeArg>,

    /// Use CLI mode instead of TUI interface
    #[arg(long)]
    pub cli_mode: bool,

    /// Layout configuration file path
    #[arg(long, value_name = "FILE", default_value = "layout.toml")]
    pub layout_config: PathBuf,

    /// Dump complete layout configuration to file
    #[arg(long, value_name = "FILE")]
    pub dump_layout: Option<PathBuf>,

    /// Subcommands
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the interactive music player (default)
    Play {
        /// Music directory to scan
        #[arg(value_name = "DIR")]
        music_dir: Option<PathBuf>,
    },
    /// Scan music library and update database
    Scan {
        /// Music directory to scan
        #[arg(value_name = "DIR")]
        music_dir: PathBuf,
        /// Force rescan of all files
        #[arg(short, long)]
        force: bool,
    },
    /// List all songs in the database
    List {
        /// Filter by artist
        #[arg(short, long)]
        artist: Option<String>,
        /// Filter by album
        #[arg(short = 'A', long)]
        album: Option<String>,
    },
    /// Search for songs
    Search {
        /// Search query
        query: String,
    },
    /// Manage playlists
    Playlist {
        #[command(subcommand)]
        action: PlaylistAction,
    },
    /// Toggle shuffle mode
    Shuffle {
        /// Enable or disable shuffle
        #[arg(value_enum)]
        mode: Option<ShuffleMode>,
    },
    /// Set repeat mode
    Repeat {
        /// Repeat mode to set
        #[arg(value_enum)]
        mode: RepeatModeArg,
    },
}

/// Playlist management actions
#[derive(Subcommand, Debug, Clone)]
pub enum PlaylistAction {
    /// Create a new playlist
    Create {
        /// Playlist name
        name: String,
        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// List all playlists
    List,
    /// Show playlist contents
    Show {
        /// Playlist name
        name: String,
    },
    /// Delete a playlist
    Delete {
        /// Playlist name
        name: String,
    },
    /// Add songs to a playlist
    Add {
        /// Playlist name
        playlist: String,
        /// Song title or path (supports multiple)
        #[arg(required = true)]
        songs: Vec<String>,
    },
    /// Remove songs from a playlist
    Remove {
        /// Playlist name
        playlist: String,
        /// Song title or path (supports multiple)
        #[arg(required = true)]
        songs: Vec<String>,
    },
    /// Play a playlist
    Play {
        /// Playlist name
        name: String,
    },
}

/// Shuffle mode for CLI
#[derive(Debug, Clone, ValueEnum)]
pub enum ShuffleMode {
    On,
    Off,
    Toggle,
}

impl Cli {
    /// Get the music directory, using platform-specific defaults if not specified
    pub fn get_music_dir(&self) -> PathBuf {
        if let Some(ref dir) = self.music_dir {
            return dir.clone();
        }

        // Check if a subcommand specifies a music directory
        if let Some(Commands::Play { music_dir: Some(ref dir) }) = self.command {
            return dir.clone();
        }

        // Use platform-specific default directories
        Self::default_music_dir()
    }

    /// Get platform-specific default music directory
    pub fn default_music_dir() -> PathBuf {
        if cfg!(target_os = "macos") {
            PathBuf::from("/Users/Shared/Music")
        } else if cfg!(target_os = "windows") {
            PathBuf::from("C:\\Users\\Public\\Music")
        } else {
            PathBuf::from("/home/music")
        }
    }

    /// Check if the specified music directory exists
    pub fn validate_music_dir(&self) -> crate::error::Result<PathBuf> {
        let music_dir = self.get_music_dir();
        
        if !music_dir.exists() {
            return Err(crate::error::LofiTurtleError::DirectoryNotFound(
                format!("Music directory '{}' does not exist", music_dir.display())
            ));
        }

        if !music_dir.is_dir() {
            return Err(crate::error::LofiTurtleError::Configuration(
                format!("'{}' is not a directory", music_dir.display())
            ));
        }

        Ok(music_dir)
    }
}
