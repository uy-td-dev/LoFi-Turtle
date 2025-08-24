use crate::domain::entities::Song;
use crate::domain::repositories::PlaylistSongRepository;
use crate::domain::value_objects::{PlaylistId, SongId, FilePath, Duration};
use crate::shared::errors::{ApplicationError, Result};
use async_trait::async_trait;
use rusqlite::{params, Connection, Result as SqliteResult, OptionalExtension};
use std::sync::{Arc, Mutex};
use tokio::task;

/// SQLite implementation of PlaylistSongRepository
pub struct SqlitePlaylistSongRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqlitePlaylistSongRepository {
    /// Create new SQLite playlist-song repository
    pub fn new(connection: Arc<Mutex<Connection>>) -> Self {
        Self { connection }
    }

    /// Initialize database schema
    pub fn initialize_schema(&self) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS playlist_songs (
                playlist_id TEXT NOT NULL,
                song_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (playlist_id, song_id),
                FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
                FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE
            )",
            [],
        ).map_err(|e| ApplicationError::Repository(
            format!("Failed to create playlist_songs table: {}", e)
        ))?;

        // Create indexes for better performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_playlist_songs_playlist_id ON playlist_songs(playlist_id)",
            [],
        ).map_err(|e| ApplicationError::Repository(
            format!("Failed to create playlist_id index: {}", e)
        ))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_playlist_songs_position ON playlist_songs(playlist_id, position)",
            [],
        ).map_err(|e| ApplicationError::Repository(
            format!("Failed to create position index: {}", e)
        ))?;

        Ok(())
    }

    /// Convert database row to Song entity
    fn row_to_song(row: &rusqlite::Row) -> SqliteResult<Song> {
        let path_str: String = row.get(1)?;
        let file_path = FilePath::new(&path_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(1, "path".to_string(), rusqlite::types::Type::Text))?;
        
        let duration_secs: i64 = row.get(5)?;
        let duration = Duration::from_seconds(duration_secs as u64);

        Song::new(
            file_path,
            row.get(2)?, // title
            row.get(3)?, // artist
            row.get(4)?, // album
            duration,
        ).map_err(|_| rusqlite::Error::InvalidColumnType(0, "song_creation".to_string(), rusqlite::types::Type::Text))
    }
}

#[async_trait]
impl PlaylistSongRepository for SqlitePlaylistSongRepository {
    async fn add_song_to_playlist(
        &self,
        playlist_id: &PlaylistId,
        song_id: &SongId,
        position: usize,
    ) -> Result<()> {
        let playlist_id = playlist_id.clone();
        let song_id = song_id.clone();
        let connection = self.connection.clone();
        
        task::spawn_blocking(move || {
            let conn = connection.lock().unwrap();
            
            // Start transaction
            let tx = conn.unchecked_transaction().map_err(|e| ApplicationError::Repository(
                format!("Failed to start transaction: {}", e)
            ))?;

            // Shift existing songs at or after this position
            tx.execute(
                "UPDATE playlist_songs SET position = position + 1 
                 WHERE playlist_id = ?1 AND position >= ?2",
                params![playlist_id.as_str(), position as i64],
            ).map_err(|e| ApplicationError::Repository(
                format!("Failed to shift song positions: {}", e)
            ))?;

            // Insert the new song
            tx.execute(
                "INSERT OR REPLACE INTO playlist_songs (playlist_id, song_id, position, added_at)
                 VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)",
                params![
                    playlist_id.as_str(),
                    song_id.as_str(),
                    position as i64
                ],
            ).map_err(|e| ApplicationError::Repository(
                format!("Failed to add song to playlist: {}", e)
            ))?;

            // Commit transaction
            tx.commit().map_err(|e| ApplicationError::Repository(
                format!("Failed to commit transaction: {}", e)
            ))?;

            Ok(())
        }).await.map_err(|e| ApplicationError::Repository(
            format!("Task execution failed: {}", e)
        ))?
    }

    async fn remove_song_from_playlist(
        &self,
        playlist_id: &PlaylistId,
        song_id: &SongId,
    ) -> Result<()> {
        let playlist_id = playlist_id.clone();
        let song_id = song_id.clone();
        let connection = self.connection.clone();
        
        task::spawn_blocking(move || {
            let conn = connection.lock().unwrap();
            
            // Start transaction
            let tx = conn.unchecked_transaction().map_err(|e| ApplicationError::Repository(
                format!("Failed to start transaction: {}", e)
            ))?;

            // Get the position of the song to be removed
            let position: Option<i64> = tx.query_row(
                "SELECT position FROM playlist_songs WHERE playlist_id = ?1 AND song_id = ?2",
                params![playlist_id.as_str(), song_id.as_str()],
                |row| row.get(0)
            ).optional().map_err(|e| ApplicationError::Repository(
                format!("Failed to get song position: {}", e)
            ))?;

            if let Some(pos) = position {
                // Remove the song
                tx.execute(
                    "DELETE FROM playlist_songs WHERE playlist_id = ?1 AND song_id = ?2",
                    params![playlist_id.as_str(), song_id.as_str()],
                ).map_err(|e| ApplicationError::Repository(
                    format!("Failed to remove song from playlist: {}", e)
                ))?;

                // Shift remaining songs down
                tx.execute(
                    "UPDATE playlist_songs SET position = position - 1 
                     WHERE playlist_id = ?1 AND position > ?2",
                    params![playlist_id.as_str(), pos],
                ).map_err(|e| ApplicationError::Repository(
                    format!("Failed to shift song positions: {}", e)
                ))?;
            }

            // Commit transaction
            tx.commit().map_err(|e| ApplicationError::Repository(
                format!("Failed to commit transaction: {}", e)
            ))?;

            Ok(())
        }).await.map_err(|e| ApplicationError::Repository(
            format!("Task execution failed: {}", e)
        ))?
    }

    async fn get_playlist_songs(&self, playlist_id: &PlaylistId) -> Result<Vec<Song>> {
        let playlist_id = playlist_id.clone();
        let connection = self.connection.clone();
        
        task::spawn_blocking(move || {
            let conn = connection.lock().unwrap();
            
            let mut stmt = conn.prepare(
                "SELECT s.id, s.path, s.title, s.artist, s.album, s.duration
                 FROM playlist_songs ps
                 JOIN songs s ON ps.song_id = s.id
                 WHERE ps.playlist_id = ?1
                 ORDER BY ps.position"
            ).map_err(|e| ApplicationError::Repository(
                format!("Failed to prepare statement: {}", e)
            ))?;

            let song_iter = stmt.query_map([playlist_id.as_str()], Self::row_to_song)
                .map_err(|e| ApplicationError::Repository(
                    format!("Failed to query playlist songs: {}", e)
                ))?;

            let mut songs = Vec::new();
            for song_result in song_iter {
                let song = song_result.map_err(|e| ApplicationError::Repository(
                    format!("Failed to parse song row: {}", e)
                ))?;
                songs.push(song);
            }

            Ok(songs)
        }).await.map_err(|e| ApplicationError::Repository(
            format!("Task execution failed: {}", e)
        ))?
    }

    async fn reorder_playlist_songs(
        &self,
        playlist_id: &PlaylistId,
        song_ids: &[SongId],
    ) -> Result<()> {
        let playlist_id = playlist_id.clone();
        let song_ids: Vec<String> = song_ids.iter().map(|id| id.as_str().to_string()).collect();
        let connection = self.connection.clone();
        
        task::spawn_blocking(move || {
            let conn = connection.lock().unwrap();
            
            // Start transaction
            let tx = conn.unchecked_transaction().map_err(|e| ApplicationError::Repository(
                format!("Failed to start transaction: {}", e)
            ))?;

            // Update positions for each song
            for (position, song_id) in song_ids.iter().enumerate() {
                tx.execute(
                    "UPDATE playlist_songs SET position = ?1 
                     WHERE playlist_id = ?2 AND song_id = ?3",
                    params![position as i64, playlist_id.as_str(), song_id],
                ).map_err(|e| ApplicationError::Repository(
                    format!("Failed to update song position: {}", e)
                ))?;
            }

            // Commit transaction
            tx.commit().map_err(|e| ApplicationError::Repository(
                format!("Failed to commit transaction: {}", e)
            ))?;

            Ok(())
        }).await.map_err(|e| ApplicationError::Repository(
            format!("Task execution failed: {}", e)
        ))?
    }

    async fn clear_playlist(&self, playlist_id: &PlaylistId) -> Result<()> {
        let playlist_id = playlist_id.clone();
        let connection = self.connection.clone();
        
        task::spawn_blocking(move || {
            let conn = connection.lock().unwrap();
            
            conn.execute(
                "DELETE FROM playlist_songs WHERE playlist_id = ?1",
                [playlist_id.as_str()],
            ).map_err(|e| ApplicationError::Repository(
                format!("Failed to clear playlist: {}", e)
            ))?;

            Ok(())
        }).await.map_err(|e| ApplicationError::Repository(
            format!("Task execution failed: {}", e)
        ))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::{SqliteSongRepository, SqlitePlaylistRepository};
    use crate::domain::entities::{Song, Playlist};
    use crate::domain::repositories::{SongRepository, PlaylistRepository, PlaylistSongRepository};
    use rusqlite::Connection;

    async fn create_test_setup() -> (SqlitePlaylistSongRepository, SqliteSongRepository, SqlitePlaylistRepository) {
        // Use in-memory database for tests to avoid permission issues
        let conn = Arc::new(Mutex::new(Connection::open(":memory:").unwrap()));
        
        let playlist_song_repo = SqlitePlaylistSongRepository::new(conn.clone());
        let song_repo = SqliteSongRepository::new(conn.clone());
        let playlist_repo = SqlitePlaylistRepository::new(conn.clone());
        
        playlist_song_repo.initialize_schema().unwrap();
        song_repo.initialize_schema().unwrap();
        playlist_repo.initialize_schema().unwrap();
        
        (playlist_song_repo, song_repo, playlist_repo)
    }

    #[tokio::test]
    async fn test_add_and_get_playlist_songs() {
        let (playlist_song_repo, song_repo, playlist_repo) = create_test_setup().await;
        
        // Create a playlist
        let playlist = Playlist::new(
            "Test Playlist".to_string(),
            None
        ).unwrap();
        playlist_repo.save(&playlist).await.unwrap();

        // Create songs
        let file_path1 = FilePath::new("/test/song1.mp3").unwrap();
        let duration1 = Duration::from_seconds(180);
        let song1 = Song::new(
            file_path1,
            "Song 1".to_string(),
            "Artist 1".to_string(),
            "Album 1".to_string(),
            duration1,
        ).unwrap();
        song_repo.save(&song1).await.unwrap();

        let file_path2 = FilePath::new("/test/song2.mp3").unwrap();
        let duration2 = Duration::from_seconds(200);
        let song2 = Song::new(
            file_path2,
            "Song 2".to_string(),
            "Artist 2".to_string(),
            "Album 2".to_string(),
            duration2,
        ).unwrap();
        song_repo.save(&song2).await.unwrap();

        // Add songs to playlist
        playlist_song_repo.add_song_to_playlist(playlist.id(), song1.id(), 0).await.unwrap();
        playlist_song_repo.add_song_to_playlist(playlist.id(), song2.id(), 1).await.unwrap();

        // Get playlist songs
        let songs = playlist_song_repo.get_playlist_songs(playlist.id()).await.unwrap();
        assert_eq!(songs.len(), 2);
        assert_eq!(songs[0].title(), "Song 1");
        assert_eq!(songs[1].title(), "Song 2");
    }

    #[tokio::test]
    async fn test_remove_song_from_playlist() {
        let (playlist_song_repo, song_repo, playlist_repo) = create_test_setup().await;
        
        // Create playlist and songs (similar setup as above)
        let playlist = Playlist::new(
            "Test Playlist".to_string(),
            None
        ).unwrap();
        playlist_repo.save(&playlist).await.unwrap();

        let file_path = FilePath::new("/test/song.mp3").unwrap();
        let duration = Duration::from_seconds(180);
        let song = Song::new(
            file_path,
            "Test Song".to_string(),
            "Test Artist".to_string(),
            "Test Album".to_string(),
            duration,
        ).unwrap();
        song_repo.save(&song).await.unwrap();

        // Add and then remove song
        playlist_song_repo.add_song_to_playlist(playlist.id(), song.id(), 0).await.unwrap();
        let songs = playlist_song_repo.get_playlist_songs(playlist.id()).await.unwrap();
        assert_eq!(songs.len(), 1);

        playlist_song_repo.remove_song_from_playlist(playlist.id(), song.id()).await.unwrap();
        let songs = playlist_song_repo.get_playlist_songs(playlist.id()).await.unwrap();
        assert_eq!(songs.len(), 0);
    }
}
