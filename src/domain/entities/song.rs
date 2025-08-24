use crate::domain::value_objects::{SongId, Duration, FilePath};
use crate::shared::errors::DomainError;
use serde::{Deserialize, Serialize};

/// Core Song entity representing a music track in the domain
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Song {
    id: SongId,
    file_path: FilePath,
    title: String,
    artist: String,
    album: String,
    duration: Duration,
}

impl Song {
    /// Create a new Song entity with validation
    pub fn new(
        file_path: FilePath,
        title: String,
        artist: String,
        album: String,
        duration: Duration,
    ) -> Result<Self, DomainError> {
        // Business rule: Title cannot be empty
        if title.trim().is_empty() {
            return Err(DomainError::InvalidSongTitle("Title cannot be empty".to_string()));
        }

        // Generate ID from file path (business rule: ID is deterministic)
        let id = SongId::from_path(&file_path);

        Ok(Self {
            id,
            file_path,
            title: title.trim().to_string(),
            artist: artist.trim().to_string(),
            album: album.trim().to_string(),
            duration,
        })
    }

    /// Get song ID
    pub fn id(&self) -> &SongId {
        &self.id
    }

    /// Get file path
    pub fn file_path(&self) -> &FilePath {
        &self.file_path
    }

    /// Get title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get artist (returns "Unknown Artist" if empty)
    pub fn artist(&self) -> &str {
        if self.artist.is_empty() {
            "Unknown Artist"
        } else {
            &self.artist
        }
    }

    /// Get album (returns "Unknown Album" if empty)
    pub fn album(&self) -> &str {
        if self.album.is_empty() {
            "Unknown Album"
        } else {
            &self.album
        }
    }

    /// Get duration
    pub fn duration(&self) -> &Duration {
        &self.duration
    }

    /// Get display name for UI (business rule: format as "Title - Artist")
    #[allow(dead_code)]
    pub fn display_name(&self) -> String {
        if self.artist().is_empty() || self.artist() == "Unknown Artist" {
            self.title.clone()
        } else {
            format!("{} - {}", self.title, self.artist())
        }
    }

    /// Update metadata (business rule: preserve ID and file path)
    #[allow(dead_code)]
    pub fn update_metadata(
        &mut self,
        title: String,
        artist: String,
        album: String,
        duration: Duration,
    ) -> Result<(), DomainError> {
        if title.trim().is_empty() {
            return Err(DomainError::InvalidSongTitle("Title cannot be empty".to_string()));
        }

        self.title = title.trim().to_string();
        self.artist = artist.trim().to_string();
        self.album = album.trim().to_string();
        self.duration = duration;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_song_creation_with_valid_data() {
        let file_path = FilePath::new("/music/song.mp3").unwrap();
        let duration = Duration::from_seconds(180);
        
        let song = Song::new(
            file_path,
            "Test Song".to_string(),
            "Test Artist".to_string(),
            "Test Album".to_string(),
            duration,
        );

        assert!(song.is_ok());
        let song = song.unwrap();
        assert_eq!(song.title(), "Test Song");
        assert_eq!(song.artist(), "Test Artist");
        assert_eq!(song.display_name(), "Test Song - Test Artist");
    }

    #[test]
    fn test_song_creation_with_empty_title_fails() {
        let file_path = FilePath::new("/music/song.mp3").unwrap();
        let duration = Duration::from_seconds(180);
        
        let result = Song::new(
            file_path,
            "".to_string(),
            "Test Artist".to_string(),
            "Test Album".to_string(),
            duration,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_artist_fallback() {
        let file_path = FilePath::new("/music/song.mp3").unwrap();
        let duration = Duration::from_seconds(180);
        
        let song = Song::new(
            file_path,
            "Test Song".to_string(),
            "".to_string(),
            "Test Album".to_string(),
            duration,
        ).unwrap();

        assert_eq!(song.artist(), "Unknown Artist");
        assert_eq!(song.display_name(), "Test Song");
    }
}
