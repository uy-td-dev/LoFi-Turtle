use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Represents a playlist containing multiple songs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub song_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Playlist {
    /// Create a new playlist with the given name
    pub fn new(name: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            song_ids: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }


    /// Get the number of songs in the playlist
    pub fn song_count(&self) -> usize {
        self.song_ids.len()
    }


    /// Get a display name for the playlist
    pub fn display_name(&self) -> String {
        format!("{} ({} songs)", self.name, self.song_count())
    }
}

/// Builder pattern for creating playlists with validation
#[derive(Default)]
pub struct PlaylistBuilder {
    name: Option<String>,
    description: Option<String>,
    song_ids: Vec<String>,
}

impl PlaylistBuilder {
    /// Create a new playlist builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the playlist name
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the playlist description
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }


    /// Build the playlist
    pub fn build(self) -> Result<Playlist, String> {
        let name = self.name.ok_or("Playlist name is required")?;
        
        if name.trim().is_empty() {
            return Err("Playlist name cannot be empty".to_string());
        }

        let mut playlist = Playlist::new(name, self.description);
        playlist.song_ids = self.song_ids;
        
        Ok(playlist)
    }
}
