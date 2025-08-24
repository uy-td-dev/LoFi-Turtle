use crate::domain::value_objects::{PlaylistId, SongId};
use crate::shared::errors::{DomainError, DomainResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Playlist entity with business rules
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Playlist {
    id: PlaylistId,
    name: String,
    description: Option<String>,
    song_ids: Vec<SongId>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Playlist {
    /// Create a new playlist with validation
    #[allow(dead_code)]
    pub fn new(name: String, description: Option<String>) -> DomainResult<Self> {
        // Business rule: Playlist name cannot be empty
        if name.trim().is_empty() {
            return Err(DomainError::InvalidPlaylistName("Name cannot be empty".to_string()));
        }

        // Business rule: Playlist name cannot be too long
        if name.len() > 100 {
            return Err(DomainError::InvalidPlaylistName("Name cannot exceed 100 characters".to_string()));
        }

        let now = Utc::now();
        Ok(Self {
            id: PlaylistId::new(),
            name: name.trim().to_string(),
            description: description.map(|d| d.trim().to_string()).filter(|d| !d.is_empty()),
            song_ids: Vec::new(),
            created_at: now,
            updated_at: now,
        })
    }

    /// Create playlist from existing data (for repository loading)
    pub fn from_existing(
        id: PlaylistId,
        name: String,
        description: Option<String>,
        song_ids: Vec<SongId>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> DomainResult<Self> {
        if name.trim().is_empty() {
            return Err(DomainError::InvalidPlaylistName("Name cannot be empty".to_string()));
        }

        Ok(Self {
            id,
            name: name.trim().to_string(),
            description: description.map(|d| d.trim().to_string()).filter(|d| !d.is_empty()),
            song_ids,
            created_at,
            updated_at,
        })
    }

    /// Get playlist ID
    pub fn id(&self) -> &PlaylistId {
        &self.id
    }

    /// Get playlist name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get playlist description
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get song IDs in order
    #[allow(dead_code)]
    pub fn song_ids(&self) -> &[SongId] {
        &self.song_ids
    }

    /// Get creation timestamp
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Get last update timestamp
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Get number of songs in playlist
    #[allow(dead_code)]
    pub fn song_count(&self) -> usize {
        self.song_ids.len()
    }

    /// Check if playlist is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.song_ids.is_empty()
    }

    /// Check if playlist contains a specific song
    #[allow(dead_code)]
    pub fn contains_song(&self, song_id: &SongId) -> bool {
        self.song_ids.contains(song_id)
    }

    /// Add song to playlist (business rule: no duplicates)
    #[allow(dead_code)]
    pub fn add_song(&mut self, song_id: SongId) -> DomainResult<()> {
        if self.contains_song(&song_id) {
            return Err(DomainError::BusinessRuleViolation(
                "Song already exists in playlist".to_string()
            ));
        }

        self.song_ids.push(song_id);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Remove song from playlist
    #[allow(dead_code)]
    pub fn remove_song(&mut self, song_id: &SongId) -> DomainResult<()> {
        let initial_len = self.song_ids.len();
        self.song_ids.retain(|id| id != song_id);
        
        if self.song_ids.len() == initial_len {
            return Err(DomainError::SongNotFound(song_id.as_str().to_string()));
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Move song to different position (for reordering)
    #[allow(dead_code)]
    pub fn move_song(&mut self, from_index: usize, to_index: usize) -> DomainResult<()> {
        if from_index >= self.song_ids.len() || to_index >= self.song_ids.len() {
            return Err(DomainError::BusinessRuleViolation(
                "Invalid song position".to_string()
            ));
        }

        let song_id = self.song_ids.remove(from_index);
        self.song_ids.insert(to_index, song_id);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Clear all songs from playlist
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.song_ids.clear();
        self.updated_at = Utc::now();
    }

    /// Update playlist metadata
    #[allow(dead_code)]
    pub fn update_metadata(&mut self, name: String, description: Option<String>) -> DomainResult<()> {
        if name.trim().is_empty() {
            return Err(DomainError::InvalidPlaylistName("Name cannot be empty".to_string()));
        }

        if name.len() > 100 {
            return Err(DomainError::InvalidPlaylistName("Name cannot exceed 100 characters".to_string()));
        }

        self.name = name.trim().to_string();
        self.description = description.map(|d| d.trim().to_string()).filter(|d| !d.is_empty());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Get display info for UI
    #[allow(dead_code)]
    pub fn display_info(&self) -> String {
        let count = self.song_count();
        let song_text = if count == 1 { "song" } else { "songs" };
        format!("{} ({} {})", self.name, count, song_text)
    }
}

/// Builder pattern for creating playlists
#[allow(dead_code)]
pub struct PlaylistBuilder {
    name: Option<String>,
    description: Option<String>,
    song_ids: Vec<SongId>,
}

impl PlaylistBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            song_ids: Vec::new(),
        }
    }

    /// Set playlist name
    #[allow(dead_code)]
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set playlist description
    #[allow(dead_code)]
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add song to playlist
    #[allow(dead_code)]
    pub fn add_song(mut self, song_id: SongId) -> Self {
        self.song_ids.push(song_id);
        self
    }

    /// Add multiple songs to playlist
    #[allow(dead_code)]
    pub fn add_songs<I>(mut self, song_ids: I) -> Self 
    where 
        I: IntoIterator<Item = SongId>
    {
        self.song_ids.extend(song_ids);
        self
    }

    /// Build the playlist
    #[allow(dead_code)]
    pub fn build(self) -> DomainResult<Playlist> {
        let name = self.name.ok_or_else(|| {
            DomainError::InvalidPlaylistName("Name is required".to_string())
        })?;

        let mut playlist = Playlist::new(name, self.description)?;
        
        // Add songs without duplicates
        for song_id in self.song_ids {
            if !playlist.contains_song(&song_id) {
                playlist.song_ids.push(song_id);
            }
        }

        Ok(playlist)
    }
}

impl Default for PlaylistBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{FilePath, SongId};

    #[test]
    fn test_playlist_creation() {
        let playlist = Playlist::new("My Playlist".to_string(), None);
        assert!(playlist.is_ok());
        
        let playlist = playlist.unwrap();
        assert_eq!(playlist.name(), "My Playlist");
        assert!(playlist.is_empty());
        assert_eq!(playlist.song_count(), 0);
    }

    #[test]
    fn test_playlist_creation_with_empty_name_fails() {
        let result = Playlist::new("".to_string(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_song_to_playlist() {
        let mut playlist = Playlist::new("Test".to_string(), None).unwrap();
        let file_path = FilePath::new("/test.mp3").unwrap();
        let song_id = SongId::from_path(&file_path);
        
        let result = playlist.add_song(song_id.clone());
        assert!(result.is_ok());
        assert!(playlist.contains_song(&song_id));
        assert_eq!(playlist.song_count(), 1);
    }

    #[test]
    fn test_add_duplicate_song_fails() {
        let mut playlist = Playlist::new("Test".to_string(), None).unwrap();
        let file_path = FilePath::new("/test.mp3").unwrap();
        let song_id = SongId::from_path(&file_path);
        
        playlist.add_song(song_id.clone()).unwrap();
        let result = playlist.add_song(song_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_playlist_builder() {
        let file_path = FilePath::new("/test.mp3").unwrap();
        let song_id = SongId::from_path(&file_path);
        
        let playlist = PlaylistBuilder::new()
            .name("Built Playlist")
            .description("Test description")
            .add_song(song_id.clone())
            .build();
            
        assert!(playlist.is_ok());
        let playlist = playlist.unwrap();
        assert_eq!(playlist.name(), "Built Playlist");
        assert!(playlist.contains_song(&song_id));
    }
}
