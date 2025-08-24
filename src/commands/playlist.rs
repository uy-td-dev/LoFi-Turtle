use crate::config::Config;
use crate::error::{LofiTurtleError, Result};
use crate::library::Database;
use crate::models::{PlaylistBuilder, Song};
use crate::commands::Command;
use crate::cli::{PlaylistAction, ShuffleMode};

/// Command for managing playlists
pub struct PlaylistCommand {
    action: PlaylistAction,
}

impl PlaylistCommand {
    /// Create a new playlist command
    pub fn new(action: PlaylistAction) -> Self {
        Self { action }
    }

    /// Create a new playlist
    fn create_playlist(&self, config: &Config, name: String, description: Option<String>) -> Result<()> {
        let db = Database::new(&config.database_path)?;
        
        // Check if playlist already exists
        if db.playlist_exists(&name)? {
            return Err(LofiTurtleError::Configuration(
                format!("Playlist '{}' already exists", name)
            ));
        }

        // Create the playlist using builder pattern
        let playlist = PlaylistBuilder::new()
            .name(name.clone())
            .description(description.unwrap_or_default())
            .build()
            .map_err(|e| LofiTurtleError::Configuration(e))?;

        db.create_playlist(&playlist)?;
        
        println!("✅ Created playlist '{}'", name);
        if let Some(desc) = &playlist.description {
            if !desc.is_empty() {
                println!("   Description: {}", desc);
            }
        }
        
        Ok(())
    }

    /// List all playlists
    fn list_playlists(&self, config: &Config) -> Result<()> {
        let db = Database::new(&config.database_path)?;
        let playlists = db.get_all_playlists()?;

        if playlists.is_empty() {
            println!("📝 No playlists found. Create one with: lofiturtle playlist create <name>");
            return Ok(());
        }

        println!("🎵 Playlists:");
        println!("─────────────────────────────────────────────────────────────");
        
        for playlist in playlists {
            println!("📁 {}", playlist.display_name());
            if let Some(description) = &playlist.description {
                if !description.is_empty() {
                    println!("   {}", description);
                }
            }
            println!("   Created: {}", playlist.created_at.format("%Y-%m-%d %H:%M"));
            if playlist.updated_at != playlist.created_at {
                println!("   Updated: {}", playlist.updated_at.format("%Y-%m-%d %H:%M"));
            }
            println!();
        }
        
        Ok(())
    }

    /// Show playlist contents
    fn show_playlist(&self, config: &Config, name: String) -> Result<()> {
        let db = Database::new(&config.database_path)?;
        
        let playlist = db.get_playlist_by_name(&name)?
            .ok_or_else(|| LofiTurtleError::Configuration(
                format!("Playlist '{}' not found", name)
            ))?;

        let songs = db.get_playlist_songs(&playlist.id)?;

        println!("🎵 Playlist: {}", playlist.name);
        if let Some(description) = &playlist.description {
            if !description.is_empty() {
                println!("📝 Description: {}", description);
            }
        }
        println!("🎼 Songs: {}", songs.len());
        println!("─────────────────────────────────────────────────────────────");

        if songs.is_empty() {
            println!("📭 This playlist is empty. Add songs with:");
            println!("   lofiturtle playlist add \"{}\" \"<song_title>\"", name);
            return Ok(());
        }

        let total_duration: u64 = songs.iter().map(|s| s.duration).sum();
        let total_minutes = total_duration / 60;
        let total_seconds = total_duration % 60;
        
        println!("⏱️  Total duration: {:02}:{:02}", total_minutes, total_seconds);
        println!();

        for (index, song) in songs.iter().enumerate() {
            println!("{}. {} - {} [{}]", 
                index + 1, 
                song.title, 
                song.artist,
                song.duration_formatted()
            );
            if !song.album.is_empty() {
                println!("   Album: {}", song.album);
            }
        }
        
        Ok(())
    }

    /// Delete a playlist
    fn delete_playlist(&self, config: &Config, name: String) -> Result<()> {
        let db = Database::new(&config.database_path)?;
        
        let playlist = db.get_playlist_by_name(&name)?
            .ok_or_else(|| LofiTurtleError::Configuration(
                format!("Playlist '{}' not found", name)
            ))?;

        // Confirm deletion
        println!("⚠️  Are you sure you want to delete playlist '{}'? (y/N)", name);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)
            .map_err(|e| LofiTurtleError::Configuration(format!("Failed to read input: {}", e)))?;

        if input.trim().to_lowercase() != "y" && input.trim().to_lowercase() != "yes" {
            println!("❌ Deletion cancelled");
            return Ok(());
        }

        db.delete_playlist(&playlist.id)?;
        println!("✅ Deleted playlist '{}'", name);
        
        Ok(())
    }

    /// Add songs to a playlist
    fn add_songs_to_playlist(&self, config: &Config, playlist_name: String, song_queries: Vec<String>) -> Result<()> {
        let db = Database::new(&config.database_path)?;
        
        let playlist = db.get_playlist_by_name(&playlist_name)?
            .ok_or_else(|| LofiTurtleError::Configuration(
                format!("Playlist '{}' not found", playlist_name)
            ))?;

        let all_songs = db.get_all_songs()?;
        let mut added_count = 0;

        for query in song_queries {
            // Find matching songs (by title, artist, or path)
            let matching_songs: Vec<&Song> = all_songs
                .iter()
                .filter(|song| {
                    song.title.to_lowercase().contains(&query.to_lowercase()) ||
                    song.artist.to_lowercase().contains(&query.to_lowercase()) ||
                    song.path.contains(&query)
                })
                .collect();

            if matching_songs.is_empty() {
                println!("❌ No songs found matching '{}'", query);
                continue;
            }

            if matching_songs.len() > 1 {
                println!("🔍 Multiple songs found for '{}'. Please be more specific:", query);
                for (i, song) in matching_songs.iter().take(5).enumerate() {
                    println!("  {}. {} - {}", i + 1, song.title, song.artist);
                }
                if matching_songs.len() > 5 {
                    println!("  ... and {} more", matching_songs.len() - 5);
                }
                continue;
            }

            let song = matching_songs[0];
            let current_songs = db.get_playlist_songs(&playlist.id)?;
            let position = current_songs.len();

            db.add_song_to_playlist(&playlist.id, &song.id, position)?;
            println!("✅ Added '{}' by {} to playlist '{}'", song.title, song.artist, playlist_name);
            added_count += 1;
        }

        if added_count > 0 {
            println!("🎵 Added {} song(s) to playlist '{}'", added_count, playlist_name);
        }
        
        Ok(())
    }

    /// Remove songs from a playlist
    fn remove_songs_from_playlist(&self, config: &Config, playlist_name: String, song_queries: Vec<String>) -> Result<()> {
        let db = Database::new(&config.database_path)?;
        
        let playlist = db.get_playlist_by_name(&playlist_name)?
            .ok_or_else(|| LofiTurtleError::Configuration(
                format!("Playlist '{}' not found", playlist_name)
            ))?;

        let playlist_songs = db.get_playlist_songs(&playlist.id)?;
        let mut removed_count = 0;

        for query in song_queries {
            // Find matching songs in the playlist
            let matching_songs: Vec<&Song> = playlist_songs
                .iter()
                .filter(|song| {
                    song.title.to_lowercase().contains(&query.to_lowercase()) ||
                    song.artist.to_lowercase().contains(&query.to_lowercase()) ||
                    song.path.contains(&query)
                })
                .collect();

            if matching_songs.is_empty() {
                println!("❌ No songs found matching '{}' in playlist '{}'", query, playlist_name);
                continue;
            }

            if matching_songs.len() > 1 {
                println!("🔍 Multiple songs found for '{}' in playlist. Please be more specific:", query);
                for (i, song) in matching_songs.iter().take(5).enumerate() {
                    println!("  {}. {} - {}", i + 1, song.title, song.artist);
                }
                continue;
            }

            let song = matching_songs[0];
            db.remove_song_from_playlist(&playlist.id, &song.id)?;
            println!("✅ Removed '{}' by {} from playlist '{}'", song.title, song.artist, playlist_name);
            removed_count += 1;
        }

        if removed_count > 0 {
            println!("🗑️  Removed {} song(s) from playlist '{}'", removed_count, playlist_name);
        }
        
        Ok(())
    }

    /// Play a playlist
    fn play_playlist(&self, config: &Config, name: String) -> Result<()> {
        let db = Database::new(&config.database_path)?;
        
        let playlist = db.get_playlist_by_name(&name)?
            .ok_or_else(|| LofiTurtleError::Configuration(
                format!("Playlist '{}' not found", name)
            ))?;

        let songs = db.get_playlist_songs(&playlist.id)?;

        if songs.is_empty() {
            return Err(LofiTurtleError::Configuration(
                format!("Playlist '{}' is empty", name)
            ));
        }

        println!("🎵 Playing playlist: {} ({} songs)", playlist.name, songs.len());
        
        // Start the TUI player with the playlist
        let play_command = crate::commands::PlayCommand::new();
        play_command.execute(config)?;
        
        Ok(())
    }
}

impl Command for PlaylistCommand {
    fn execute(&self, config: &Config) -> Result<()> {
        match &self.action {
            PlaylistAction::Create { name, description } => {
                self.create_playlist(config, name.clone(), description.clone())
            }
            PlaylistAction::List => {
                self.list_playlists(config)
            }
            PlaylistAction::Show { name } => {
                self.show_playlist(config, name.clone())
            }
            PlaylistAction::Delete { name } => {
                self.delete_playlist(config, name.clone())
            }
            PlaylistAction::Add { playlist, songs } => {
                self.add_songs_to_playlist(config, playlist.clone(), songs.clone())
            }
            PlaylistAction::Remove { playlist, songs } => {
                self.remove_songs_from_playlist(config, playlist.clone(), songs.clone())
            }
            PlaylistAction::Play { name } => {
                self.play_playlist(config, name.clone())
            }
        }
    }

    fn description(&self) -> &'static str {
        "Manage music playlists"
    }
}

/// Command for controlling shuffle mode
pub struct ShuffleCommand {
    mode: Option<ShuffleMode>,
}

impl ShuffleCommand {
    pub fn new(mode: Option<ShuffleMode>) -> Self {
        Self { mode }
    }
}

impl Command for ShuffleCommand {
    fn execute(&self, _config: &Config) -> Result<()> {
        match &self.mode {
            Some(ShuffleMode::On) => {
                println!("🔀 Shuffle mode enabled");
            }
            Some(ShuffleMode::Off) => {
                println!("➡️  Shuffle mode disabled");
            }
            Some(ShuffleMode::Toggle) | None => {
                println!("🔀 Toggling shuffle mode");
            }
        }
        
        // Note: The actual shuffle state will be managed by the PlaybackManager
        // in the TUI interface. This command is mainly for CLI feedback.
        
        Ok(())
    }

    fn description(&self) -> &'static str {
        "Control shuffle playback mode"
    }
}

/// Command for controlling repeat mode
pub struct RepeatCommand {
    mode: crate::cli::RepeatModeArg,
}

impl RepeatCommand {
    pub fn new(mode: crate::cli::RepeatModeArg) -> Self {
        Self { mode }
    }
}

impl Command for RepeatCommand {
    fn execute(&self, _config: &Config) -> Result<()> {
        let mode_str = match self.mode {
            crate::cli::RepeatModeArg::None => "off",
            crate::cli::RepeatModeArg::Single => "single song",
            crate::cli::RepeatModeArg::Playlist => "playlist",
        };
        
        let icon = match self.mode {
            crate::cli::RepeatModeArg::None => "⏭",
            crate::cli::RepeatModeArg::Single => "🔂",
            crate::cli::RepeatModeArg::Playlist => "🔁",
        };
        
        println!("{} Repeat mode set to: {}", icon, mode_str);
        
        // Note: The actual repeat state will be managed by the PlaybackManager
        // in the TUI interface. This command is mainly for CLI feedback.
        
        Ok(())
    }

    fn description(&self) -> &'static str {
        "Control repeat playback mode"
    }
}
