#![allow(dead_code)]
use crate::domain::entities::Playlist;
use crate::domain::repositories::{PlaylistRepository, SongRepository, PlaylistSongRepository};
use crate::domain::value_objects::{PlaylistId, SongId};
use crate::shared::errors::{ApplicationError, Result};
use std::sync::Arc;

/// Use case for creating a new playlist
pub struct CreatePlaylistUseCase {
    playlist_repository: Arc<dyn PlaylistRepository>,
}

impl CreatePlaylistUseCase {
    pub fn new(playlist_repository: Arc<dyn PlaylistRepository>) -> Self {
        Self { playlist_repository }
    }

    /// Execute the use case to create a playlist
    pub async fn execute(&self, request: CreatePlaylistRequest) -> Result<CreatePlaylistResponse> {
        // Check if playlist with same name already exists
        if self.playlist_repository.exists_by_name(&request.name).await? {
            return Err(ApplicationError::ValidationFailed(
                format!("Playlist '{}' already exists", request.name)
            ));
        }

        // Create new playlist entity
        let playlist = Playlist::new(request.name, request.description)
            .map_err(ApplicationError::Domain)?;

        // Save to repository
        self.playlist_repository.save(&playlist).await?;

        Ok(CreatePlaylistResponse {
            playlist_id: playlist.id().clone(),
        })
    }
}

/// Use case for adding songs to a playlist
pub struct AddSongToPlaylistUseCase {
    playlist_repository: Arc<dyn PlaylistRepository>,
    song_repository: Arc<dyn SongRepository>,
    playlist_song_repository: Arc<dyn PlaylistSongRepository>,
}

impl AddSongToPlaylistUseCase {
    pub fn new(
        playlist_repository: Arc<dyn PlaylistRepository>,
        song_repository: Arc<dyn SongRepository>,
        playlist_song_repository: Arc<dyn PlaylistSongRepository>,
    ) -> Self {
        Self {
            playlist_repository,
            song_repository,
            playlist_song_repository,
        }
    }

    /// Execute the use case
    pub async fn execute(&self, request: AddSongToPlaylistRequest) -> Result<AddSongToPlaylistResponse> {
        // Verify playlist exists
        let mut playlist = self.playlist_repository
            .find_by_id(&request.playlist_id)
            .await?
            .ok_or_else(|| ApplicationError::UseCaseFailed(
                format!("Playlist not found: {}", request.playlist_id.as_str())
            ))?;

        // Verify song exists
        let _song = self.song_repository
            .find_by_id(&request.song_id)
            .await?
            .ok_or_else(|| ApplicationError::UseCaseFailed(
                format!("Song not found: {}", request.song_id.as_str())
            ))?;

        // Add song to playlist entity (business rules applied)
        playlist.add_song(request.song_id.clone())
            .map_err(ApplicationError::Domain)?;

        // Update playlist in repository
        self.playlist_repository.save(&playlist).await?;

        // Add to playlist-song relationship
        self.playlist_song_repository
            .add_song_to_playlist(&request.playlist_id, &request.song_id, playlist.song_count() - 1)
            .await?;

        Ok(AddSongToPlaylistResponse {
            playlist_id: request.playlist_id,
            song_id: request.song_id,
        })
    }
}

/// Use case for removing songs from a playlist
pub struct RemoveSongFromPlaylistUseCase {
    playlist_repository: Arc<dyn PlaylistRepository>,
    playlist_song_repository: Arc<dyn PlaylistSongRepository>,
}

impl RemoveSongFromPlaylistUseCase {
    pub fn new(
        playlist_repository: Arc<dyn PlaylistRepository>,
        playlist_song_repository: Arc<dyn PlaylistSongRepository>,
    ) -> Self {
        Self {
            playlist_repository,
            playlist_song_repository,
        }
    }

    /// Execute the use case
    pub async fn execute(&self, request: RemoveSongFromPlaylistRequest) -> Result<RemoveSongFromPlaylistResponse> {
        // Get playlist
        let mut playlist = self.playlist_repository
            .find_by_id(&request.playlist_id)
            .await?
            .ok_or_else(|| ApplicationError::UseCaseFailed(
                format!("Playlist not found: {}", request.playlist_id.as_str())
            ))?;

        // Remove song from playlist entity
        playlist.remove_song(&request.song_id)
            .map_err(ApplicationError::Domain)?;

        // Update playlist in repository
        self.playlist_repository.save(&playlist).await?;

        // Remove from playlist-song relationship
        self.playlist_song_repository
            .remove_song_from_playlist(&request.playlist_id, &request.song_id)
            .await?;

        Ok(RemoveSongFromPlaylistResponse {
            playlist_id: request.playlist_id,
            song_id: request.song_id,
        })
    }
}

/// Use case for getting playlist with songs
pub struct GetPlaylistWithSongsUseCase {
    playlist_repository: Arc<dyn PlaylistRepository>,
    playlist_song_repository: Arc<dyn PlaylistSongRepository>,
}

impl GetPlaylistWithSongsUseCase {
    pub fn new(
        playlist_repository: Arc<dyn PlaylistRepository>,
        playlist_song_repository: Arc<dyn PlaylistSongRepository>,
    ) -> Self {
        Self {
            playlist_repository,
            playlist_song_repository,
        }
    }

    /// Execute the use case
    pub async fn execute(&self, request: GetPlaylistWithSongsRequest) -> Result<GetPlaylistWithSongsResponse> {
        // Get playlist
        let playlist = self.playlist_repository
            .find_by_id(&request.playlist_id)
            .await?
            .ok_or_else(|| ApplicationError::UseCaseFailed(
                format!("Playlist not found: {}", request.playlist_id.as_str())
            ))?;

        // Get songs for playlist
        let songs = self.playlist_song_repository
            .get_playlist_songs(&request.playlist_id)
            .await?;

        Ok(GetPlaylistWithSongsResponse {
            playlist,
            songs,
        })
    }
}

/// Use case for deleting a playlist
pub struct DeletePlaylistUseCase {
    playlist_repository: Arc<dyn PlaylistRepository>,
    playlist_song_repository: Arc<dyn PlaylistSongRepository>,
}

impl DeletePlaylistUseCase {
    pub fn new(
        playlist_repository: Arc<dyn PlaylistRepository>,
        playlist_song_repository: Arc<dyn PlaylistSongRepository>,
    ) -> Self {
        Self {
            playlist_repository,
            playlist_song_repository,
        }
    }

    /// Execute the use case
    pub async fn execute(&self, request: DeletePlaylistRequest) -> Result<DeletePlaylistResponse> {
        // Verify playlist exists
        let _playlist = self.playlist_repository
            .find_by_id(&request.playlist_id)
            .await?
            .ok_or_else(|| ApplicationError::UseCaseFailed(
                format!("Playlist not found: {}", request.playlist_id.as_str())
            ))?;

        // Clear all songs from playlist
        self.playlist_song_repository
            .clear_playlist(&request.playlist_id)
            .await?;

        // Delete playlist
        self.playlist_repository
            .delete(&request.playlist_id)
            .await?;

        Ok(DeletePlaylistResponse {
            playlist_id: request.playlist_id,
        })
    }
}

// Request/Response DTOs

#[derive(Debug, Clone)]
pub struct CreatePlaylistRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreatePlaylistResponse {
    pub playlist_id: PlaylistId,
}

#[derive(Debug, Clone)]
pub struct AddSongToPlaylistRequest {
    pub playlist_id: PlaylistId,
    pub song_id: SongId,
}

#[derive(Debug, Clone)]
pub struct AddSongToPlaylistResponse {
    pub playlist_id: PlaylistId,
    pub song_id: SongId,
}

#[derive(Debug, Clone)]
pub struct RemoveSongFromPlaylistRequest {
    pub playlist_id: PlaylistId,
    pub song_id: SongId,
}

#[derive(Debug, Clone)]
pub struct RemoveSongFromPlaylistResponse {
    pub playlist_id: PlaylistId,
    pub song_id: SongId,
}

#[derive(Debug, Clone)]
pub struct GetPlaylistWithSongsRequest {
    pub playlist_id: PlaylistId,
}

#[derive(Debug, Clone)]
pub struct GetPlaylistWithSongsResponse {
    pub playlist: Playlist,
    pub songs: Vec<crate::domain::entities::Song>,
}

#[derive(Debug, Clone)]
pub struct DeletePlaylistRequest {
    pub playlist_id: PlaylistId,
}

#[derive(Debug, Clone)]
pub struct DeletePlaylistResponse {
    pub playlist_id: PlaylistId,
}
