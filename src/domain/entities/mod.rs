/// Domain entities module
/// 
/// Contains the core business entities that represent the main concepts
/// in the music player domain.

pub mod song;
pub mod playlist;

pub use song::Song;
pub use playlist::Playlist;
