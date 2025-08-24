#![allow(dead_code)]
use crate::domain::entities::Song;
use crate::domain::repositories::SongRepository;
use crate::domain::value_objects::{SongId, FilePath, Duration};
use crate::shared::errors::{ApplicationError, Result};
use std::sync::Arc;

/// Use case for adding a new song to the library
pub struct AddSongUseCase {
    song_repository: Arc<dyn SongRepository>,
}

impl AddSongUseCase {
    pub fn new(song_repository: Arc<dyn SongRepository>) -> Self {
        Self { song_repository }
    }

    /// Execute the use case to add a song
    pub async fn execute(&self, request: AddSongRequest) -> Result<AddSongResponse> {
        // Check if song already exists
        if self.song_repository.exists_by_path(&request.file_path).await? {
            return Ok(AddSongResponse {
                song_id: SongId::from_path(&request.file_path),
                was_created: false,
            });
        }

        // Create new song entity
        let song = Song::new(
            request.file_path,
            request.title,
            request.artist,
            request.album,
            request.duration,
        ).map_err(ApplicationError::Domain)?;

        // Save to repository
        self.song_repository.save(&song).await?;

        Ok(AddSongResponse {
            song_id: song.id().clone(),
            was_created: true,
        })
    }
}

/// Use case for searching songs
pub struct SearchSongsUseCase {
    song_repository: Arc<dyn SongRepository>,
}

impl SearchSongsUseCase {
    pub fn new(song_repository: Arc<dyn SongRepository>) -> Self {
        Self { song_repository }
    }

    /// Execute the search
    pub async fn execute(&self, request: SearchSongsRequest) -> Result<SearchSongsResponse> {
        let songs = if request.query.trim().is_empty() {
            self.song_repository.find_all().await?
        } else {
            self.song_repository.search(&request.query).await?
        };

        Ok(SearchSongsResponse { songs })
    }
}

/// Use case for getting song details
pub struct GetSongUseCase {
    song_repository: Arc<dyn SongRepository>,
}

impl GetSongUseCase {
    pub fn new(song_repository: Arc<dyn SongRepository>) -> Self {
        Self { song_repository }
    }

    /// Execute the use case
    pub async fn execute(&self, request: GetSongRequest) -> Result<GetSongResponse> {
        let song = self.song_repository
            .find_by_id(&request.song_id)
            .await?
            .ok_or_else(|| ApplicationError::UseCaseFailed(
                format!("Song not found: {}", request.song_id.as_str())
            ))?;

        Ok(GetSongResponse { song })
    }
}

/// Use case for removing a song from the library
pub struct RemoveSongUseCase {
    song_repository: Arc<dyn SongRepository>,
}

impl RemoveSongUseCase {
    pub fn new(song_repository: Arc<dyn SongRepository>) -> Self {
        Self { song_repository }
    }

    /// Execute the use case
    pub async fn execute(&self, request: RemoveSongRequest) -> Result<RemoveSongResponse> {
        // Verify song exists
        let _song = self.song_repository
            .find_by_id(&request.song_id)
            .await?
            .ok_or_else(|| ApplicationError::UseCaseFailed(
                format!("Song not found: {}", request.song_id.as_str())
            ))?;

        // Remove from repository
        self.song_repository.delete(&request.song_id).await?;

        Ok(RemoveSongResponse {
            song_id: request.song_id,
        })
    }
}

// Request/Response DTOs (Data Transfer Objects)

#[derive(Debug, Clone)]
pub struct AddSongRequest {
    pub file_path: FilePath,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: Duration,
}

#[derive(Debug, Clone)]
pub struct AddSongResponse {
    pub song_id: SongId,
    pub was_created: bool,
}

#[derive(Debug, Clone)]
pub struct SearchSongsRequest {
    pub query: String,
}

#[derive(Debug, Clone)]
pub struct SearchSongsResponse {
    pub songs: Vec<Song>,
}

#[derive(Debug, Clone)]
pub struct GetSongRequest {
    pub song_id: SongId,
}

#[derive(Debug, Clone)]
pub struct GetSongResponse {
    pub song: Song,
}

#[derive(Debug, Clone)]
pub struct RemoveSongRequest {
    pub song_id: SongId,
}

#[derive(Debug, Clone)]
pub struct RemoveSongResponse {
    pub song_id: SongId,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::SongRepository;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mock repository for testing
    struct MockSongRepository {
        songs: Mutex<HashMap<SongId, Song>>,
    }

    impl MockSongRepository {
        fn new() -> Self {
            Self {
                songs: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl SongRepository for MockSongRepository {
        async fn save(&self, song: &Song) -> Result<()> {
            let mut songs = self.songs.lock().unwrap();
            songs.insert(song.id().clone(), song.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: &SongId) -> Result<Option<Song>> {
            let songs = self.songs.lock().unwrap();
            Ok(songs.get(id).cloned())
        }

        async fn find_by_path(&self, path: &FilePath) -> Result<Option<Song>> {
            let songs = self.songs.lock().unwrap();
            Ok(songs.values().find(|s| s.file_path() == path).cloned())
        }

        async fn find_all(&self) -> Result<Vec<Song>> {
            let songs = self.songs.lock().unwrap();
            Ok(songs.values().cloned().collect())
        }

        async fn search(&self, query: &str) -> Result<Vec<Song>> {
            let songs = self.songs.lock().unwrap();
            let query_lower = query.to_lowercase();
            Ok(songs.values()
                .filter(|s| {
                    s.title().to_lowercase().contains(&query_lower) ||
                    s.artist().to_lowercase().contains(&query_lower) ||
                    s.album().to_lowercase().contains(&query_lower)
                })
                .cloned()
                .collect())
        }

        async fn exists_by_path(&self, path: &FilePath) -> Result<bool> {
            Ok(self.find_by_path(path).await?.is_some())
        }

        async fn delete(&self, id: &SongId) -> Result<()> {
            let mut songs = self.songs.lock().unwrap();
            songs.remove(id);
            Ok(())
        }

        async fn find_by_ids(&self, ids: &[SongId]) -> Result<Vec<Song>> {
            let songs = self.songs.lock().unwrap();
            Ok(ids.iter()
                .filter_map(|id| songs.get(id).cloned())
                .collect())
        }

        async fn clear_all(&self) -> Result<()> {
            let mut songs = self.songs.lock().unwrap();
            songs.clear();
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_add_song_use_case() {
        let repository = Arc::new(MockSongRepository::new());
        let use_case = AddSongUseCase::new(repository);

        let request = AddSongRequest {
            file_path: FilePath::new("/test/song.mp3").unwrap(),
            title: "Test Song".to_string(),
            artist: "Test Artist".to_string(),
            album: "Test Album".to_string(),
            duration: Duration::from_seconds(180),
        };

        let response = use_case.execute(request).await.unwrap();
        assert!(response.was_created);
    }

    #[tokio::test]
    async fn test_search_songs_use_case() {
        let repository = Arc::new(MockSongRepository::new());
        let use_case_add = AddSongUseCase::new(repository.clone());
        let use_case_search = SearchSongsUseCase::new(repository);

        // Add a test song
        let add_request = AddSongRequest {
            file_path: FilePath::new("/test/song.mp3").unwrap(),
            title: "Test Song".to_string(),
            artist: "Test Artist".to_string(),
            album: "Test Album".to_string(),
            duration: Duration::from_seconds(180),
        };
        use_case_add.execute(add_request).await.unwrap();

        // Search for it
        let search_request = SearchSongsRequest {
            query: "Test".to_string(),
        };
        let response = use_case_search.execute(search_request).await.unwrap();
        assert_eq!(response.songs.len(), 1);
        assert_eq!(response.songs[0].title(), "Test Song");
    }
}
