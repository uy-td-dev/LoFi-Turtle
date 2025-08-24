use thiserror::Error;

/// Custom error types for the LofiTurtle music player
#[derive(Error, Debug)]
pub enum LofiTurtleError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Audio playback error: {0}")]
    AudioPlayback(String),

    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    #[allow(dead_code)] // Future feature: music library operations
    #[error("Music library error: {0}")]
    MusicLibrary(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Terminal interface error: {0}")]
    Terminal(String),

    #[error("Audio file format not supported: {0}")]
    UnsupportedFormat(String),

    #[error("Music directory not found: {0}")]
    DirectoryNotFound(String),

    #[allow(dead_code)] // Future feature: command validation
    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Channel communication error: {0}")]
    ChannelError(String),

}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, LofiTurtleError>;

impl From<Box<dyn std::error::Error + Send + Sync>> for LofiTurtleError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        LofiTurtleError::AudioPlayback(err.to_string())
    }
}

impl From<std::sync::mpsc::SendError<crate::audio::PlayerCommand>> for LofiTurtleError {
    fn from(err: std::sync::mpsc::SendError<crate::audio::PlayerCommand>) -> Self {
        LofiTurtleError::ChannelError(format!("Failed to send player command: {}", err))
    }
}
