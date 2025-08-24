#![allow(dead_code)]
/// Domain layer - Core business logic and entities
/// 
/// This layer contains:
/// - Business entities (Song, Playlist)
/// - Value objects (SongId, FilePath, Duration, Volume, etc.)
/// - Domain services and business rules
/// - Repository interfaces (abstractions)
/// 
/// The domain layer is the heart of Clean Architecture and should be
/// independent of any external concerns (UI, database, frameworks).

pub mod entities;
pub mod value_objects;
pub mod repositories;

// Re-export commonly used types for convenience
// Re-export core domain types for convenience
// Note: These are available but may not all be used in current implementation
