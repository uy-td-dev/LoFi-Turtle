/// Data Transfer Objects (DTOs) for the application layer
/// 
/// DTOs are used to transfer data between layers without exposing
/// internal domain structures.

use crate::domain::entities::{Song, Playlist};
// Value objects are imported but may not all be used in current DTOs
use serde::{Deserialize, Serialize};

/// DTO for song information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongDto {
    pub id: String,
    pub file_path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_seconds: u64,
}

impl From<Song> for SongDto {
    fn from(song: Song) -> Self {
        Self {
            id: song.id().as_str().to_string(),
            file_path: song.file_path().as_str().to_string(),
            title: song.title().to_string(),
            artist: song.artist().to_string(),
            album: song.album().to_string(),
            duration_seconds: song.duration().total_seconds(),
        }
    }
}

/// DTO for playlist information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Playlist> for PlaylistDto {
    fn from(playlist: Playlist) -> Self {
        Self {
            id: playlist.id().as_str().to_string(),
            name: playlist.name().to_string(),
            description: playlist.description().map(|s| s.to_string()),
            created_at: playlist.created_at().to_rfc3339(),
            updated_at: playlist.updated_at().to_rfc3339(),
        }
    }
}

/// DTO for playlist with songs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistWithSongsDto {
    pub playlist: PlaylistDto,
    pub songs: Vec<SongDto>,
}

/// DTO for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultDto {
    pub songs: Vec<SongDto>,
    pub total_count: usize,
    pub query: String,
}

/// DTO for library statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryStatsDto {
    pub total_songs: usize,
    pub total_playlists: usize,
    pub total_duration_seconds: u64,
    pub unique_artists: usize,
    pub unique_albums: usize,
}
