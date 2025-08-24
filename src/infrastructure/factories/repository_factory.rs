use crate::domain::repositories::{SongRepository, PlaylistRepository, PlaylistSongRepository};
use crate::infrastructure::repositories::{
    SqliteSongRepository, SqlitePlaylistRepository, SqlitePlaylistSongRepository
};
use crate::shared::errors::{ApplicationError, Result};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

/// Factory for creating repository instances with dependency injection
/// 
/// This factory implements the Factory pattern and provides a centralized
/// way to create and configure repository instances.
pub struct RepositoryFactory {
    connection: Arc<Mutex<Connection>>,
}

impl RepositoryFactory {
    /// Create new repository factory with database connection
    pub fn new(database_path: &str) -> Result<Self> {
        let connection = Connection::open(database_path)
            .map_err(|e| ApplicationError::Repository(
                format!("Failed to open database: {}", e)
            ))?;

        let factory = Self {
            connection: Arc::new(Mutex::new(connection)),
        };

        // Initialize all schemas
        factory.initialize_schemas()?;

        Ok(factory)
    }

    /// Create new in-memory repository factory for testing
    pub fn new_in_memory() -> Result<Self> {
        let connection = Connection::open_in_memory()
            .map_err(|e| ApplicationError::Repository(
                format!("Failed to create in-memory database: {}", e)
            ))?;

        let factory = Self {
            connection: Arc::new(Mutex::new(connection)),
        };

        // Initialize all schemas
        factory.initialize_schemas()?;

        Ok(factory)
    }

    /// Initialize all database schemas
    fn initialize_schemas(&self) -> Result<()> {
        let song_repo = self.create_song_repository();
        song_repo.initialize_schema()?;

        let playlist_repo = self.create_playlist_repository();
        playlist_repo.initialize_schema()?;

        let playlist_song_repo = self.create_playlist_song_repository();
        playlist_song_repo.initialize_schema()?;

        Ok(())
    }

    /// Create song repository instance
    pub fn create_song_repository(&self) -> SqliteSongRepository {
        SqliteSongRepository::new(self.connection.clone())
    }

    /// Create playlist repository instance
    pub fn create_playlist_repository(&self) -> SqlitePlaylistRepository {
        SqlitePlaylistRepository::new(self.connection.clone())
    }

    /// Create playlist-song repository instance
    pub fn create_playlist_song_repository(&self) -> SqlitePlaylistSongRepository {
        SqlitePlaylistSongRepository::new(self.connection.clone())
    }

    /// Create song repository as trait object for dependency injection
    pub fn create_song_repository_arc(&self) -> Arc<dyn SongRepository> {
        Arc::new(self.create_song_repository())
    }

    /// Create playlist repository as trait object for dependency injection
    pub fn create_playlist_repository_arc(&self) -> Arc<dyn PlaylistRepository> {
        Arc::new(self.create_playlist_repository())
    }

    /// Create playlist-song repository as trait object for dependency injection
    pub fn create_playlist_song_repository_arc(&self) -> Arc<dyn PlaylistSongRepository> {
        Arc::new(self.create_playlist_song_repository())
    }

    /// Create all repositories as a bundle for convenience
    pub fn create_all_repositories(&self) -> RepositoryBundle {
        RepositoryBundle {
            song_repository: self.create_song_repository_arc(),
            playlist_repository: self.create_playlist_repository_arc(),
            playlist_song_repository: self.create_playlist_song_repository_arc(),
        }
    }

    /// Get database connection for advanced operations
    pub fn get_connection(&self) -> Arc<Mutex<Connection>> {
        self.connection.clone()
    }

    /// Execute database migrations if needed
    pub fn migrate(&self) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        
        // Check current schema version
        let version: i32 = conn.query_row(
            "PRAGMA user_version",
            [],
            |row| row.get(0)
        ).unwrap_or(0);

        match version {
            0 => {
                // Initial schema - already created by initialize_schemas
                conn.execute("PRAGMA user_version = 1", [])
                    .map_err(|e| ApplicationError::Repository(
                        format!("Failed to set schema version: {}", e)
                    ))?;
            }
            // Add future migrations here
            _ => {
                // Schema is up to date
            }
        }

        Ok(())
    }

    /// Perform database maintenance operations
    pub fn maintain(&self) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        
        // Analyze tables for query optimization
        conn.execute("ANALYZE", [])
            .map_err(|e| ApplicationError::Repository(
                format!("Failed to analyze database: {}", e)
            ))?;

        // Vacuum database to reclaim space
        conn.execute("VACUUM", [])
            .map_err(|e| ApplicationError::Repository(
                format!("Failed to vacuum database: {}", e)
            ))?;

        Ok(())
    }
}

/// Bundle of all repository instances for dependency injection
pub struct RepositoryBundle {
    pub song_repository: Arc<dyn SongRepository>,
    pub playlist_repository: Arc<dyn PlaylistRepository>,
    pub playlist_song_repository: Arc<dyn PlaylistSongRepository>,
}

impl RepositoryBundle {
    /// Create repositories from factory
    pub fn from_factory(factory: &RepositoryFactory) -> Self {
        factory.create_all_repositories()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::Song;
    use crate::domain::value_objects::{FilePath, Duration};

    #[tokio::test]
    async fn test_repository_factory() {
        let factory = RepositoryFactory::new_in_memory().unwrap();
        let bundle = factory.create_all_repositories();

        // Test song repository
        let file_path = FilePath::new("/test/song.mp3").unwrap();
        let duration = Duration::from_seconds(180);
        let song = Song::new(
            file_path,
            "Test Song".to_string(),
            "Test Artist".to_string(),
            "Test Album".to_string(),
            duration,
        ).unwrap();

        bundle.song_repository.save(&song).await.unwrap();
        let found_song = bundle.song_repository.find_by_id(song.id()).await.unwrap();
        assert!(found_song.is_some());
        assert_eq!(found_song.unwrap().title(), "Test Song");
    }

    #[test]
    fn test_factory_migration() {
        let factory = RepositoryFactory::new_in_memory().unwrap();
        factory.migrate().unwrap();
        
        // Should be able to migrate multiple times without error
        factory.migrate().unwrap();
    }

    #[test]
    fn test_factory_maintenance() {
        let factory = RepositoryFactory::new_in_memory().unwrap();
        factory.maintain().unwrap();
    }
}
