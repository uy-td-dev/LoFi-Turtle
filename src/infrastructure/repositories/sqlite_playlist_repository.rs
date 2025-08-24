use crate::domain::entities::Playlist;
use crate::domain::repositories::PlaylistRepository;
use crate::domain::value_objects::PlaylistId;
use crate::shared::errors::{ApplicationError, Result};
use async_trait::async_trait;
use rusqlite::{params, Connection, Result as SqliteResult};
use std::sync::{Arc, Mutex};
use tokio::task;
use chrono::{DateTime, Utc};

/// SQLite implementation of PlaylistRepository
pub struct SqlitePlaylistRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqlitePlaylistRepository {
    /// Create new SQLite playlist repository
    pub fn new(connection: Arc<Mutex<Connection>>) -> Self {
        Self { connection }
    }

    /// Initialize database schema
    pub fn initialize_schema(&self) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS playlists (
                id TEXT PRIMARY KEY,
                name TEXT UNIQUE NOT NULL,
                description TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        ).map_err(|e| ApplicationError::Repository(
            format!("Failed to create playlists table: {}", e)
        ))?;

        // Create indexes for better performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_playlists_name ON playlists(name)",
            [],
        ).map_err(|e| ApplicationError::Repository(
            format!("Failed to create name index: {}", e)
        ))?;

        Ok(())
    }

    /// Convert database row to Playlist entity
    fn row_to_playlist(row: &rusqlite::Row) -> SqliteResult<Playlist> {
        let id_str: String = row.get(0)?;
        let playlist_id = PlaylistId::from_string(id_str);

        let name: String = row.get(1)?;
        let description: Option<String> = row.get(2)?;
        let created_at_str: String = row.get(3)?;
        let updated_at_str: String = row.get(4)?;

        // Parse timestamps
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .or_else(|_| DateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S"))
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .or_else(|_| DateTime::parse_from_str(&updated_at_str, "%Y-%m-%d %H:%M:%S"))
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        // Create playlist using from_existing method
        let playlist = Playlist::from_existing(
            playlist_id,
            name,
            description,
            Vec::new(), // Empty song list - will be populated separately
            created_at,
            updated_at,
        ).map_err(|_| rusqlite::Error::InvalidColumnType(0, "playlist_creation".to_string(), rusqlite::types::Type::Text))?;
        
        Ok(playlist)
    }
}

#[async_trait]
impl PlaylistRepository for SqlitePlaylistRepository {
    async fn save(&self, playlist: &Playlist) -> Result<()> {
        let playlist = playlist.clone();
        let connection = self.connection.clone();
        
        task::spawn_blocking(move || {
            let conn = connection.lock().unwrap();
            
            conn.execute(
                "INSERT OR REPLACE INTO playlists (id, name, description, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    playlist.id().as_str(),
                    playlist.name(),
                    playlist.description(),
                    playlist.created_at().to_rfc3339(),
                    playlist.updated_at().to_rfc3339()
                ],
            ).map_err(|e| ApplicationError::Repository(
                format!("Failed to save playlist: {}", e)
            ))?;

            Ok(())
        }).await.map_err(|e| ApplicationError::Repository(
            format!("Task execution failed: {}", e)
        ))?
    }

    async fn find_by_id(&self, id: &PlaylistId) -> Result<Option<Playlist>> {
        let id = id.clone();
        let connection = self.connection.clone();
        
        task::spawn_blocking(move || {
            let conn = connection.lock().unwrap();
            
            let mut stmt = conn.prepare(
                "SELECT id, name, description, created_at, updated_at FROM playlists WHERE id = ?1"
            ).map_err(|e| ApplicationError::Repository(
                format!("Failed to prepare statement: {}", e)
            ))?;

            let playlist_result = stmt.query_row([id.as_str()], Self::row_to_playlist);
            
            match playlist_result {
                Ok(playlist) => Ok(Some(playlist)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(ApplicationError::Repository(
                    format!("Failed to find playlist by id: {}", e)
                )),
            }
        }).await.map_err(|e| ApplicationError::Repository(
            format!("Task execution failed: {}", e)
        ))?
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Playlist>> {
        let name = name.to_string();
        let connection = self.connection.clone();
        
        task::spawn_blocking(move || {
            let conn = connection.lock().unwrap();
            
            let mut stmt = conn.prepare(
                "SELECT id, name, description, created_at, updated_at FROM playlists WHERE name = ?1"
            ).map_err(|e| ApplicationError::Repository(
                format!("Failed to prepare statement: {}", e)
            ))?;

            let playlist_result = stmt.query_row([&name], Self::row_to_playlist);
            
            match playlist_result {
                Ok(playlist) => Ok(Some(playlist)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(ApplicationError::Repository(
                    format!("Failed to find playlist by name: {}", e)
                )),
            }
        }).await.map_err(|e| ApplicationError::Repository(
            format!("Task execution failed: {}", e)
        ))?
    }

    async fn find_all(&self) -> Result<Vec<Playlist>> {
        let connection = self.connection.clone();
        
        task::spawn_blocking(move || {
            let conn = connection.lock().unwrap();
            
            let mut stmt = conn.prepare(
                "SELECT id, name, description, created_at, updated_at FROM playlists ORDER BY name"
            ).map_err(|e| ApplicationError::Repository(
                format!("Failed to prepare statement: {}", e)
            ))?;

            let playlist_iter = stmt.query_map([], Self::row_to_playlist)
                .map_err(|e| ApplicationError::Repository(
                    format!("Failed to query playlists: {}", e)
                ))?;

            let mut playlists = Vec::new();
            for playlist_result in playlist_iter {
                let playlist = playlist_result.map_err(|e| ApplicationError::Repository(
                    format!("Failed to parse playlist row: {}", e)
                ))?;
                playlists.push(playlist);
            }

            Ok(playlists)
        }).await.map_err(|e| ApplicationError::Repository(
            format!("Task execution failed: {}", e)
        ))?
    }

    async fn delete(&self, id: &PlaylistId) -> Result<()> {
        let id = id.clone();
        let connection = self.connection.clone();
        
        task::spawn_blocking(move || {
            let conn = connection.lock().unwrap();
            
            conn.execute(
                "DELETE FROM playlists WHERE id = ?1",
                [id.as_str()],
            ).map_err(|e| ApplicationError::Repository(
                format!("Failed to delete playlist: {}", e)
            ))?;

            Ok(())
        }).await.map_err(|e| ApplicationError::Repository(
            format!("Task execution failed: {}", e)
        ))?
    }

    async fn exists_by_name(&self, name: &str) -> Result<bool> {
        let name = name.to_string();
        let connection = self.connection.clone();
        
        task::spawn_blocking(move || {
            let conn = connection.lock().unwrap();
            
            let mut stmt = conn.prepare(
                "SELECT COUNT(*) FROM playlists WHERE name = ?1"
            ).map_err(|e| ApplicationError::Repository(
                format!("Failed to prepare statement: {}", e)
            ))?;

            let count: i64 = stmt.query_row([&name], |row| row.get(0))
                .map_err(|e| ApplicationError::Repository(
                    format!("Failed to check playlist existence: {}", e)
                ))?;

            Ok(count > 0)
        }).await.map_err(|e| ApplicationError::Repository(
            format!("Task execution failed: {}", e)
        ))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::PlaylistRepository;
    use rusqlite::Connection;

    async fn create_test_repository() -> SqlitePlaylistRepository {
        // Use in-memory database for tests to avoid permission issues
        let conn = Connection::open(":memory:").unwrap();
        let repo = SqlitePlaylistRepository::new(Arc::new(Mutex::new(conn)));
        repo.initialize_schema().unwrap();
        repo
    }

    #[tokio::test]
    async fn test_save_and_find_playlist() {
        let repo = create_test_repository().await;
        
        let playlist = Playlist::new(
            "Test Playlist".to_string(),
            Some("A test playlist".to_string())
        ).unwrap();

        // Save playlist
        repo.save(&playlist).await.unwrap();

        // Find by ID
        let found_playlist = repo.find_by_id(playlist.id()).await.unwrap();
        assert!(found_playlist.is_some());
        assert_eq!(found_playlist.unwrap().name(), "Test Playlist");

        // Find by name
        let found_by_name = repo.find_by_name("Test Playlist").await.unwrap();
        assert!(found_by_name.is_some());
        assert_eq!(found_by_name.unwrap().name(), "Test Playlist");
    }

    #[tokio::test]
    async fn test_playlist_existence() {
        let repo = create_test_repository().await;
        
        let playlist = Playlist::new(
            "Test Playlist".to_string(),
            None
        ).unwrap();

        // Should not exist initially
        assert!(!repo.exists_by_name("Test Playlist").await.unwrap());

        // Save playlist
        repo.save(&playlist).await.unwrap();

        // Should exist now
        assert!(repo.exists_by_name("Test Playlist").await.unwrap());
    }

    #[tokio::test]
    async fn test_delete_playlist() {
        let repo = create_test_repository().await;
        
        let playlist = Playlist::new(
            "Test Playlist".to_string(),
            None
        ).unwrap();

        // Save and verify
        repo.save(&playlist).await.unwrap();
        assert!(repo.exists_by_name("Test Playlist").await.unwrap());

        // Delete and verify
        repo.delete(playlist.id()).await.unwrap();
        assert!(!repo.exists_by_name("Test Playlist").await.unwrap());
    }
}
