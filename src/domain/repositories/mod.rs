use crate::domain::entities::{Song, Playlist};
use crate::domain::value_objects::{SongId, PlaylistId, FilePath};
use crate::shared::errors::ApplicationError;
use async_trait::async_trait;

/// Repository interface for Song entities (Dependency Inversion Principle)
#[async_trait]
pub trait SongRepository: Send + Sync {
    /// Save a song to the repository
    async fn save(&self, song: &Song) -> Result<(), ApplicationError>;
    
    /// Find song by ID
    async fn find_by_id(&self, id: &SongId) -> Result<Option<Song>, ApplicationError>;
    
    /// Find song by file path
    async fn find_by_path(&self, path: &FilePath) -> Result<Option<Song>, ApplicationError>;
    
    /// Get all songs
    async fn find_all(&self) -> Result<Vec<Song>, ApplicationError>;
    
    /// Search songs by query (title, artist, album)
    async fn search(&self, query: &str) -> Result<Vec<Song>, ApplicationError>;
    
    /// Check if song exists by path
    async fn exists_by_path(&self, path: &FilePath) -> Result<bool, ApplicationError>;
    
    /// Delete song by ID
    async fn delete(&self, id: &SongId) -> Result<(), ApplicationError>;
    
    /// Get songs by IDs (for playlist loading)
    async fn find_by_ids(&self, ids: &[SongId]) -> Result<Vec<Song>, ApplicationError>;
    
    /// Clear all songs (for force rescan)
    async fn clear_all(&self) -> Result<(), ApplicationError>;
}

/// Repository interface for Playlist entities
#[async_trait]
pub trait PlaylistRepository: Send + Sync {
    /// Save a playlist to the repository
    async fn save(&self, playlist: &Playlist) -> Result<(), ApplicationError>;
    
    /// Find playlist by ID
    async fn find_by_id(&self, id: &PlaylistId) -> Result<Option<Playlist>, ApplicationError>;
    
    /// Find playlist by name
    async fn find_by_name(&self, name: &str) -> Result<Option<Playlist>, ApplicationError>;
    
    /// Get all playlists
    async fn find_all(&self) -> Result<Vec<Playlist>, ApplicationError>;
    
    /// Delete playlist by ID
    async fn delete(&self, id: &PlaylistId) -> Result<(), ApplicationError>;
    
    /// Check if playlist exists by name
    async fn exists_by_name(&self, name: &str) -> Result<bool, ApplicationError>;
}

/// Repository interface for managing playlist-song relationships
#[async_trait]
pub trait PlaylistSongRepository: Send + Sync {
    /// Add song to playlist at specific position
    async fn add_song_to_playlist(
        &self, 
        playlist_id: &PlaylistId, 
        song_id: &SongId, 
        position: usize
    ) -> Result<(), ApplicationError>;
    
    /// Remove song from playlist
    async fn remove_song_from_playlist(
        &self, 
        playlist_id: &PlaylistId, 
        song_id: &SongId
    ) -> Result<(), ApplicationError>;
    
    /// Get songs for a playlist in order
    async fn get_playlist_songs(&self, playlist_id: &PlaylistId) -> Result<Vec<Song>, ApplicationError>;
    
    /// Reorder songs in playlist
    async fn reorder_playlist_songs(
        &self, 
        playlist_id: &PlaylistId, 
        song_ids: &[SongId]
    ) -> Result<(), ApplicationError>;
    
    /// Clear all songs from playlist
    async fn clear_playlist(&self, playlist_id: &PlaylistId) -> Result<(), ApplicationError>;
}

/// Unit of Work pattern for transactional operations
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    /// Begin transaction
    async fn begin(&self) -> Result<(), ApplicationError>;
    
    /// Commit transaction
    async fn commit(&self) -> Result<(), ApplicationError>;
    
    /// Rollback transaction
    async fn rollback(&self) -> Result<(), ApplicationError>;
    
    /// Execute operation within transaction
    async fn execute_in_transaction<F, R>(&self, operation: F) -> Result<R, ApplicationError>
    where
        F: FnOnce() -> Result<R, ApplicationError> + Send,
        R: Send;
}
