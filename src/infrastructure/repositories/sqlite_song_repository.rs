use crate::domain::entities::Song;
use crate::domain::repositories::SongRepository;
use crate::domain::value_objects::{SongId, FilePath, Duration};
use crate::shared::errors::{ApplicationError, Result};
use async_trait::async_trait;
use rusqlite::{params, Connection, Result as SqliteResult};
use std::sync::{Arc, Mutex};
use tokio::task;

/// SQLite implementation of SongRepository
pub struct SqliteSongRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteSongRepository {
    /// Create new SQLite song repository
    pub fn new(connection: Arc<Mutex<Connection>>) -> Self {
        Self { connection }
    }

    /// Initialize database schema
    pub fn initialize_schema(&self) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS songs (
                id TEXT PRIMARY KEY,
                path TEXT UNIQUE NOT NULL,
                title TEXT NOT NULL,
                artist TEXT NOT NULL,
                album TEXT NOT NULL,
                duration INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        ).map_err(|e| ApplicationError::Repository(
            format!("Failed to create songs table: {}", e)
        ))?;

        // Create indexes for better performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_songs_title ON songs(title)",
            [],
        ).map_err(|e| ApplicationError::Repository(
            format!("Failed to create title index: {}", e)
        ))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_songs_artist ON songs(artist)",
            [],
        ).map_err(|e| ApplicationError::Repository(
            format!("Failed to create artist index: {}", e)
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
impl SongRepository for SqliteSongRepository {
    async fn save(&self, song: &Song) -> Result<()> {
        let song = song.clone();
        let connection = self.connection.clone();
        
        task::spawn_blocking(move || {
            let conn = connection.lock().unwrap();
            
            conn.execute(
                "INSERT OR REPLACE INTO songs (id, path, title, artist, album, duration, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)",
                params![
                    song.id().as_str(),
                    song.file_path().as_str(),
                    song.title(),
                    song.artist(),
                    song.album(),
                    song.duration().total_seconds() as i64
                ],
            ).map_err(|e| ApplicationError::Repository(
                format!("Failed to save song: {}", e)
            ))?;

            Ok(())
        }).await.map_err(|e| ApplicationError::Repository(
            format!("Task execution failed: {}", e)
        ))?
    }

    async fn find_by_id(&self, id: &SongId) -> Result<Option<Song>> {
        let id = id.clone();
        let connection = self.connection.clone();
        
        task::spawn_blocking(move || {
            let conn = connection.lock().unwrap();
            
            let mut stmt = conn.prepare(
                "SELECT id, path, title, artist, album, duration FROM songs WHERE id = ?1"
            ).map_err(|e| ApplicationError::Repository(
                format!("Failed to prepare statement: {}", e)
            ))?;

            let song_result = stmt.query_row([id.as_str()], Self::row_to_song);
            
            match song_result {
                Ok(song) => Ok(Some(song)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(ApplicationError::Repository(
                    format!("Failed to find song by id: {}", e)
                )),
            }
        }).await.map_err(|e| ApplicationError::Repository(
            format!("Task execution failed: {}", e)
        ))?
    }

    async fn find_by_path(&self, path: &FilePath) -> Result<Option<Song>> {
        let path = path.clone();
        let connection = self.connection.clone();
        
        task::spawn_blocking(move || {
            let conn = connection.lock().unwrap();
            
            let mut stmt = conn.prepare(
                "SELECT id, path, title, artist, album, duration FROM songs WHERE path = ?1"
            ).map_err(|e| ApplicationError::Repository(
                format!("Failed to prepare statement: {}", e)
            ))?;

            let song_result = stmt.query_row([path.as_str()], Self::row_to_song);
            
            match song_result {
                Ok(song) => Ok(Some(song)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(ApplicationError::Repository(
                    format!("Failed to find song by path: {}", e)
                )),
            }
        }).await.map_err(|e| ApplicationError::Repository(
            format!("Task execution failed: {}", e)
        ))?
    }

    async fn find_all(&self) -> Result<Vec<Song>> {
        let connection = self.connection.clone();
        
        task::spawn_blocking(move || {
            let conn = connection.lock().unwrap();
            
            let mut stmt = conn.prepare(
                "SELECT id, path, title, artist, album, duration FROM songs ORDER BY title, artist"
            ).map_err(|e| ApplicationError::Repository(
                format!("Failed to prepare statement: {}", e)
            ))?;

            let song_iter = stmt.query_map([], Self::row_to_song)
                .map_err(|e| ApplicationError::Repository(
                    format!("Failed to query songs: {}", e)
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

    async fn search(&self, query: &str) -> Result<Vec<Song>> {
        let query = query.to_string();
        let connection = self.connection.clone();
        
        task::spawn_blocking(move || {
            let conn = connection.lock().unwrap();
            let search_pattern = format!("%{}%", query.to_lowercase());
            
            let mut stmt = conn.prepare(
                "SELECT id, path, title, artist, album, duration FROM songs 
                 WHERE LOWER(title) LIKE ?1 OR LOWER(artist) LIKE ?1 OR LOWER(album) LIKE ?1
                 ORDER BY title, artist"
            ).map_err(|e| ApplicationError::Repository(
                format!("Failed to prepare search statement: {}", e)
            ))?;

            let song_iter = stmt.query_map([&search_pattern], Self::row_to_song)
                .map_err(|e| ApplicationError::Repository(
                    format!("Failed to search songs: {}", e)
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

    async fn exists_by_path(&self, path: &FilePath) -> Result<bool> {
        let path = path.clone();
        let connection = self.connection.clone();
        
        task::spawn_blocking(move || {
            let conn = connection.lock().unwrap();
            
            let mut stmt = conn.prepare(
                "SELECT COUNT(*) FROM songs WHERE path = ?1"
            ).map_err(|e| ApplicationError::Repository(
                format!("Failed to prepare statement: {}", e)
            ))?;

            let count: i64 = stmt.query_row([path.as_str()], |row| row.get(0))
                .map_err(|e| ApplicationError::Repository(
                    format!("Failed to check song existence: {}", e)
                ))?;

            Ok(count > 0)
        }).await.map_err(|e| ApplicationError::Repository(
            format!("Task execution failed: {}", e)
        ))?
    }

    async fn delete(&self, id: &SongId) -> Result<()> {
        let id = id.clone();
        let connection = self.connection.clone();
        
        task::spawn_blocking(move || {
            let conn = connection.lock().unwrap();
            
            conn.execute(
                "DELETE FROM songs WHERE id = ?1",
                [id.as_str()],
            ).map_err(|e| ApplicationError::Repository(
                format!("Failed to delete song: {}", e)
            ))?;

            Ok(())
        }).await.map_err(|e| ApplicationError::Repository(
            format!("Task execution failed: {}", e)
        ))?
    }

    async fn find_by_ids(&self, ids: &[SongId]) -> Result<Vec<Song>> {
        let ids: Vec<String> = ids.iter().map(|id| id.as_str().to_string()).collect();
        let connection = self.connection.clone();
        
        task::spawn_blocking(move || {
            let conn = connection.lock().unwrap();
            
            if ids.is_empty() {
                return Ok(Vec::new());
            }

            // Create placeholders for IN clause
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let query = format!(
                "SELECT id, path, title, artist, album, duration FROM songs WHERE id IN ({}) ORDER BY title, artist",
                placeholders
            );

            let mut stmt = conn.prepare(&query)
                .map_err(|e| ApplicationError::Repository(
                    format!("Failed to prepare statement: {}", e)
                ))?;

            let song_iter = stmt.query_map(
                rusqlite::params_from_iter(ids.iter()),
                Self::row_to_song
            ).map_err(|e| ApplicationError::Repository(
                format!("Failed to query songs by ids: {}", e)
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

    async fn clear_all(&self) -> Result<()> {
        let connection = self.connection.clone();
        
        task::spawn_blocking(move || {
            let conn = connection.lock().unwrap();
            
            conn.execute("DELETE FROM songs", [])
                .map_err(|e| ApplicationError::Repository(
                    format!("Failed to clear all songs: {}", e)
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
    use rusqlite::Connection;

    async fn create_test_repository() -> SqliteSongRepository {
        // Use in-memory database for tests to avoid permission issues
        let conn = Connection::open(":memory:").unwrap();
        let repo = SqliteSongRepository::new(Arc::new(Mutex::new(conn)));
        repo.initialize_schema().unwrap();
        repo
    }

    #[tokio::test]
    async fn test_save_and_find_song() {
        let repo = create_test_repository().await;
        
        let file_path = FilePath::new("/test/song.mp3").unwrap();
        let duration = Duration::from_seconds(180);
        let song = Song::new(
            file_path.clone(),
            "Test Song".to_string(),
            "Test Artist".to_string(),
            "Test Album".to_string(),
            duration,
        ).unwrap();

        // Save song
        repo.save(&song).await.unwrap();

        // Find by ID
        let found_song = repo.find_by_id(song.id()).await.unwrap();
        assert!(found_song.is_some());
        assert_eq!(found_song.unwrap().title(), "Test Song");

        // Find by path
        let found_by_path = repo.find_by_path(&file_path).await.unwrap();
        assert!(found_by_path.is_some());
        assert_eq!(found_by_path.unwrap().title(), "Test Song");
    }

    #[tokio::test]
    async fn test_search_songs() {
        let repo = create_test_repository().await;
        
        let file_path = FilePath::new("/test/song.mp3").unwrap();
        let duration = Duration::from_seconds(180);
        let song = Song::new(
            file_path,
            "Test Song".to_string(),
            "Test Artist".to_string(),
            "Test Album".to_string(),
            duration,
        ).unwrap();

        repo.save(&song).await.unwrap();

        // Search by title
        let results = repo.search("Test").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title(), "Test Song");

        // Search by artist
        let results = repo.search("Artist").await.unwrap();
        assert_eq!(results.len(), 1);
    }
}
