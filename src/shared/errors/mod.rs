use thiserror::Error;

/// Domain-specific errors following Clean Architecture principles
#[derive(Error, Debug, Clone, PartialEq)]
pub enum DomainError {
    #[error("Invalid song title: {0}")]
    InvalidSongTitle(String),
    
    #[error("Invalid file path: {0}")]
    InvalidFilePath(String),
    
    #[error("Invalid duration: {0}")]
    InvalidDuration(String),
    
    #[error("Invalid playlist name: {0}")]
    InvalidPlaylistName(String),
    
    
    #[error("Song not found: {0}")]
    SongNotFound(String),
    
    #[error("Invalid volume: must be between 0.0 and 1.0, got {0}")]
    InvalidVolume(f32),
    
    #[error("Business rule violation: {0}")]
    BusinessRuleViolation(String),
}

/// Application layer errors
#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),
    
    #[error("Repository error: {0}")]
    Repository(String),
    
    
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Use case failed: {0}")]
    UseCaseFailed(String),
}

/// Infrastructure layer errors
#[derive(Error, Debug)]
pub enum InfrastructureError {
    #[error("Infrastructure error: {0}")]
    #[allow(dead_code)] // Reserved for future infrastructure errors
    General(String),
}

/// Presentation layer errors
#[derive(Error, Debug)]
pub enum PresentationError {
    #[error("Application error: {0}")]
    Application(#[from] ApplicationError),
    
    #[error("Presentation error: {0}")]
    #[allow(dead_code)] // Reserved for future presentation errors
    General(String),
}

/// Main application result type
pub type Result<T> = std::result::Result<T, ApplicationError>;

/// Domain result type
pub type DomainResult<T> = std::result::Result<T, DomainError>;

