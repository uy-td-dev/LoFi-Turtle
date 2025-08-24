use crate::application::use_cases::*;
use crate::domain::repositories::{SongRepository, PlaylistRepository, PlaylistSongRepository};
use crate::domain::entities::{Song, Playlist};
use crate::domain::value_objects::{SongId, PlaylistId, FilePath, Duration};
use crate::shared::errors::Result;
use std::sync::Arc;

/// Application service for music library operations
/// 
/// This service orchestrates multiple use cases and provides a higher-level
/// interface for the presentation layer. It follows the Facade pattern.
pub struct MusicLibraryService {
    // Use cases
    add_song_use_case: AddSongUseCase,
    search_songs_use_case: SearchSongsUseCase,
    get_song_use_case: GetSongUseCase,
    remove_song_use_case: RemoveSongUseCase,
    
    create_playlist_use_case: CreatePlaylistUseCase,
    add_song_to_playlist_use_case: AddSongToPlaylistUseCase,
    remove_song_from_playlist_use_case: RemoveSongFromPlaylistUseCase,
    get_playlist_with_songs_use_case: GetPlaylistWithSongsUseCase,
    delete_playlist_use_case: DeletePlaylistUseCase,
}

impl MusicLibraryService {
    /// Create new music library service with dependency injection
    pub fn new(
        song_repository: Arc<dyn SongRepository>,
        playlist_repository: Arc<dyn PlaylistRepository>,
        playlist_song_repository: Arc<dyn PlaylistSongRepository>,
    ) -> Self {
        Self {
            add_song_use_case: AddSongUseCase::new(song_repository.clone()),
            search_songs_use_case: SearchSongsUseCase::new(song_repository.clone()),
            get_song_use_case: GetSongUseCase::new(song_repository.clone()),
            remove_song_use_case: RemoveSongUseCase::new(song_repository.clone()),
            
            create_playlist_use_case: CreatePlaylistUseCase::new(playlist_repository.clone()),
            add_song_to_playlist_use_case: AddSongToPlaylistUseCase::new(
                playlist_repository.clone(),
                song_repository.clone(),
                playlist_song_repository.clone(),
            ),
            remove_song_from_playlist_use_case: RemoveSongFromPlaylistUseCase::new(
                playlist_repository.clone(),
                playlist_song_repository.clone(),
            ),
            get_playlist_with_songs_use_case: GetPlaylistWithSongsUseCase::new(
                playlist_repository.clone(),
                playlist_song_repository.clone(),
            ),
            delete_playlist_use_case: DeletePlaylistUseCase::new(
                playlist_repository,
                playlist_song_repository,
            ),
        }
    }

    /// Add a new song to the library
    pub async fn add_song(
        &self,
        file_path: FilePath,
        title: String,
        artist: String,
        album: String,
        duration: Duration,
    ) -> Result<SongId> {
        let request = AddSongRequest {
            file_path,
            title,
            artist,
            album,
            duration,
        };

        let response = self.add_song_use_case.execute(request).await?;
        Ok(response.song_id)
    }

    /// Search for songs in the library
    pub async fn search_songs(&self, query: String) -> Result<Vec<Song>> {
        let request = SearchSongsRequest { query };
        let response = self.search_songs_use_case.execute(request).await?;
        Ok(response.songs)
    }

    /// Get all songs in the library
    pub async fn get_all_songs(&self) -> Result<Vec<Song>> {
        self.search_songs(String::new()).await
    }

    /// Get song by ID
    pub async fn get_song(&self, song_id: SongId) -> Result<Song> {
        let request = GetSongRequest { song_id };
        let response = self.get_song_use_case.execute(request).await?;
        Ok(response.song)
    }

    /// Remove song from library
    pub async fn remove_song(&self, song_id: SongId) -> Result<()> {
        let request = RemoveSongRequest { song_id };
        self.remove_song_use_case.execute(request).await?;
        Ok(())
    }

    /// Create a new playlist
    pub async fn create_playlist(&self, name: String, description: Option<String>) -> Result<PlaylistId> {
        let request = CreatePlaylistRequest { name, description };
        let response = self.create_playlist_use_case.execute(request).await?;
        Ok(response.playlist_id)
    }

    /// Add song to playlist
    pub async fn add_song_to_playlist(&self, playlist_id: PlaylistId, song_id: SongId) -> Result<()> {
        let request = AddSongToPlaylistRequest { playlist_id, song_id };
        self.add_song_to_playlist_use_case.execute(request).await?;
        Ok(())
    }

    /// Remove song from playlist
    pub async fn remove_song_from_playlist(&self, playlist_id: PlaylistId, song_id: SongId) -> Result<()> {
        let request = RemoveSongFromPlaylistRequest { playlist_id, song_id };
        self.remove_song_from_playlist_use_case.execute(request).await?;
        Ok(())
    }

    /// Get playlist with its songs
    pub async fn get_playlist_with_songs(&self, playlist_id: PlaylistId) -> Result<(Playlist, Vec<Song>)> {
        let request = GetPlaylistWithSongsRequest { playlist_id };
        let response = self.get_playlist_with_songs_use_case.execute(request).await?;
        Ok((response.playlist, response.songs))
    }

    /// Delete playlist
    pub async fn delete_playlist(&self, playlist_id: PlaylistId) -> Result<()> {
        let request = DeletePlaylistRequest { playlist_id };
        self.delete_playlist_use_case.execute(request).await?;
        Ok(())
    }

    /// Batch add multiple songs (useful for library scanning)
    pub async fn batch_add_songs(&self, songs_data: Vec<SongData>) -> Result<BatchAddResult> {
        let mut added_count = 0;
        let mut updated_count = 0;
        let mut errors = Vec::new();

        for song_data in songs_data {
            let file_path_clone = song_data.file_path.clone();
            let request = AddSongRequest {
                file_path: song_data.file_path,
                title: song_data.title,
                artist: song_data.artist,
                album: song_data.album,
                duration: song_data.duration,
            };

            match self.add_song_use_case.execute(request).await {
                Ok(response) => {
                    if response.was_created {
                        added_count += 1;
                    } else {
                        updated_count += 1;
                    }
                }
                Err(e) => {
                    errors.push(BatchError {
                        file_path: file_path_clone,
                        error: e.to_string(),
                    });
                }
            }
        }

        Ok(BatchAddResult {
            added_count,
            updated_count,
            errors,
        })
    }
}

/// Data structure for batch song addition
#[derive(Debug, Clone)]
pub struct SongData {
    pub file_path: FilePath,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: Duration,
}

/// Result of batch song addition
#[derive(Debug)]
pub struct BatchAddResult {
    pub added_count: usize,
    pub updated_count: usize,
    pub errors: Vec<BatchError>,
}

/// Error information for batch operations
#[derive(Debug)]
pub struct BatchError {
    pub file_path: FilePath,
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::{SongRepository, PlaylistRepository, PlaylistSongRepository};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mock repositories for testing
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

    struct MockPlaylistRepository;
    struct MockPlaylistSongRepository;

    #[async_trait]
    impl PlaylistRepository for MockPlaylistRepository {
        async fn save(&self, _playlist: &Playlist) -> Result<()> { Ok(()) }
        async fn find_by_id(&self, _id: &PlaylistId) -> Result<Option<Playlist>> { Ok(None) }
        async fn find_by_name(&self, _name: &str) -> Result<Option<Playlist>> { Ok(None) }
        async fn find_all(&self) -> Result<Vec<Playlist>> { Ok(Vec::new()) }
        async fn delete(&self, _id: &PlaylistId) -> Result<()> { Ok(()) }
        async fn exists_by_name(&self, _name: &str) -> Result<bool> { Ok(false) }
    }

    #[async_trait]
    impl PlaylistSongRepository for MockPlaylistSongRepository {
        async fn add_song_to_playlist(&self, _playlist_id: &PlaylistId, _song_id: &SongId, _position: usize) -> Result<()> { Ok(()) }
        async fn remove_song_from_playlist(&self, _playlist_id: &PlaylistId, _song_id: &SongId) -> Result<()> { Ok(()) }
        async fn get_playlist_songs(&self, _playlist_id: &PlaylistId) -> Result<Vec<Song>> { Ok(Vec::new()) }
        async fn reorder_playlist_songs(&self, _playlist_id: &PlaylistId, _song_ids: &[SongId]) -> Result<()> { Ok(()) }
        async fn clear_playlist(&self, _playlist_id: &PlaylistId) -> Result<()> { Ok(()) }
    }

    #[tokio::test]
    async fn test_music_library_service() {
        let song_repo = Arc::new(MockSongRepository::new());
        let playlist_repo = Arc::new(MockPlaylistRepository);
        let playlist_song_repo = Arc::new(MockPlaylistSongRepository);

        let service = MusicLibraryService::new(song_repo, playlist_repo, playlist_song_repo);

        let file_path = FilePath::new("/test/song.mp3").unwrap();
        let duration = Duration::from_seconds(180);

        let song_id = service.add_song(
            file_path,
            "Test Song".to_string(),
            "Test Artist".to_string(),
            "Test Album".to_string(),
            duration,
        ).await.unwrap();

        let song = service.get_song(song_id).await.unwrap();
        assert_eq!(song.title(), "Test Song");

        let songs = service.search_songs("Test".to_string()).await.unwrap();
        assert_eq!(songs.len(), 1);
    }
}
