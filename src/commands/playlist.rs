use crate::commands::Command;
use crate::config::Config;
use crate::error::Result;
use crate::library::Database;
use crate::models::{PlaylistBuilder, RepeatMode};
use crate::cli::{PlaylistAction, ShuffleMode, RepeatModeArg};

pub struct PlaylistCommand {
    action: PlaylistAction,
}

impl PlaylistCommand {
    pub fn new(action: PlaylistAction) -> Self {
        Self { action }
    }
}

impl Command for PlaylistCommand {
    fn execute(&self, config: &Config) -> Result<()> {
        let database = Database::new(&config.database_path)?;

        match &self.action {
            PlaylistAction::List => {
                let playlists = database.get_all_playlists()?;
                if playlists.is_empty() {
                    println!("No playlists found.");
                } else {
                    println!("Found {} playlists:", playlists.len());
                    println!("{:<30} | {:<10} | {:<30}", "Name", "Songs", "Description");
                    println!("{:-<30}-+-{:-<10}-+-{:-<30}", "", "", "");

                    for playlist in playlists {
                        println!("{:<30} | {:<10} | {:<30}",
                            playlist.name,
                            playlist.song_count(),
                            playlist.description.clone().unwrap_or_default()
                        );
                    }
                }
            }
            PlaylistAction::Create { name, description } => {
                let playlist = PlaylistBuilder::new()
                    .name(name)
                    .description(description.clone().unwrap_or_default())
                    .build()
                    .map_err(|e| crate::error::LofiTurtleError::Configuration(e))?;

                database.create_playlist(&playlist)?;
                println!("Created playlist '{}'", name);
            }
            PlaylistAction::Delete { name } => {
                if let Some(playlist) = database.get_playlist_by_name(name)? {
                    database.delete_playlist(&playlist.id)?;
                    println!("Deleted playlist '{}'", name);
                } else {
                    println!("Playlist '{}' not found.", name);
                }
            }
            PlaylistAction::Add { playlist: playlist_name, songs } => {
                if let Some(playlist) = database.get_playlist_by_name(playlist_name)? {
                    for song_query in songs {
                        // Find song by path or title
                        let found_songs = database.search_songs(song_query)?;
                        if found_songs.is_empty() {
                            println!("No songs found matching '{}'", song_query);
                        } else if found_songs.len() > 1 {
                            println!("Found multiple songs matching '{}'. Please be more specific.", song_query);
                            for (i, s) in found_songs.iter().enumerate().take(5) {
                                println!("{}. {} - {}", i + 1, s.title, s.artist);
                            }
                        } else {
                            let song_to_add = &found_songs[0];
                            // Add to end of playlist
                            let current_songs = database.get_playlist_songs(&playlist.id)?;
                            database.add_song_to_playlist(&playlist.id, &song_to_add.id, current_songs.len())?;
                            println!("Added '{}' to playlist '{}'", song_to_add.title, playlist_name);
                        }
                    }
                } else {
                    println!("Playlist '{}' not found.", playlist_name);
                }
            }
            PlaylistAction::Remove { playlist: playlist_name, songs } => {
                if let Some(playlist) = database.get_playlist_by_name(playlist_name)? {
                    for song_query in songs {
                        // Find song in playlist
                        let playlist_songs = database.get_playlist_songs(&playlist.id)?;
                        let song_to_remove = playlist_songs.iter().find(|s|
                            s.title.to_lowercase().contains(&song_query.to_lowercase()) ||
                            s.path.contains(song_query)
                        );

                        if let Some(s) = song_to_remove {
                            database.remove_song_from_playlist(&playlist.id, &s.id)?;
                            println!("Removed '{}' from playlist '{}'", s.title, playlist_name);
                        } else {
                            println!("Song matching '{}' not found in playlist '{}'", song_query, playlist_name);
                        }
                    }
                } else {
                    println!("Playlist '{}' not found.", playlist_name);
                }
            }
            PlaylistAction::Show { name } => {
                if let Some(playlist) = database.get_playlist_by_name(name)? {
                    println!("Playlist: {}", playlist.name);
                    if let Some(desc) = &playlist.description {
                        println!("Description: {}", desc);
                    }
                    println!("Songs: {}", playlist.song_count());
                    println!();

                    let songs = database.get_playlist_songs(&playlist.id)?;
                    if songs.is_empty() {
                        println!("(Empty playlist)");
                    } else {
                        for (i, song) in songs.iter().enumerate() {
                            println!("{}. {} - {} ({})", i + 1, song.title, song.artist, song.duration_formatted());
                        }
                    }
                } else {
                    println!("Playlist '{}' not found.", name);
                }
            }
            PlaylistAction::Play { name } => {
                // This is a special case that delegates to PlayCommand
                // In a real CLI, we might want to handle this differently
                // For now, we'll just print instructions
                println!("To play a playlist, use the interactive mode or: lofiturtle play --playlist '{}'", name);
            }
        }

        Ok(())
    }

    fn description(&self) -> &'static str {
        "Manage playlists (create, delete, add/remove songs)"
    }
}

pub struct ShuffleCommand {
    mode: ShuffleMode,
}

impl ShuffleCommand {
    pub fn new(mode: ShuffleMode) -> Self {
        Self { mode }
    }
}

impl Command for ShuffleCommand {
    fn execute(&self, _config: &Config) -> Result<()> {
        // This command modifies persistent settings
        let mut settings = crate::config::PersistentSettings::load();

        match self.mode {
            ShuffleMode::On => {
                settings.shuffle = true;
                println!("Shuffle mode enabled");
            }
            ShuffleMode::Off => {
                settings.shuffle = false;
                println!("Shuffle mode disabled");
            }
            ShuffleMode::Toggle => {
                settings.shuffle = !settings.shuffle;
                println!("Shuffle mode {}", if settings.shuffle { "enabled" } else { "disabled" });
            }
        }

        settings.save()?;
        Ok(())
    }

    fn description(&self) -> &'static str {
        "Configure shuffle mode"
    }
}

pub struct RepeatCommand {
    mode: RepeatModeArg,
}

impl RepeatCommand {
    pub fn new(mode: RepeatModeArg) -> Self {
        Self { mode }
    }
}

impl Command for RepeatCommand {
    fn execute(&self, _config: &Config) -> Result<()> {
        // This command modifies persistent settings
        let mut settings = crate::config::PersistentSettings::load();

        match self.mode {
            RepeatModeArg::None => {
                settings.repeat_mode = RepeatMode::None;
                println!("Repeat mode: None");
            }
            RepeatModeArg::Single => {
                settings.repeat_mode = RepeatMode::Single;
                println!("Repeat mode: Single Song");
            }
            RepeatModeArg::Playlist => {
                settings.repeat_mode = RepeatMode::Playlist;
                println!("Repeat mode: Playlist");
            }
        }

        settings.save()?;
        Ok(())
    }

    fn description(&self) -> &'static str {
        "Configure repeat mode"
    }
}
