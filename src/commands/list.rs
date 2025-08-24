use crate::commands::Command;
use crate::config::Config;
use crate::error::Result;
use crate::library::Database;

/// Command to list songs in the database with optional filtering
pub struct ListCommand {
    artist_filter: Option<String>,
    album_filter: Option<String>,
}

impl ListCommand {
    pub fn new(artist_filter: Option<String>, album_filter: Option<String>) -> Self {
        Self {
            artist_filter,
            album_filter,
        }
    }
}

impl Command for ListCommand {
    fn execute(&self, config: &Config) -> Result<()> {
        let database = Database::new(&config.database_path)?;
        let songs = database.get_all_songs()?;
        
        let filtered_songs: Vec<_> = songs
            .iter()
            .filter(|song| {
                if let Some(ref artist) = self.artist_filter {
                    if !song.artist.to_lowercase().contains(&artist.to_lowercase()) {
                        return false;
                    }
                }
                if let Some(ref album) = self.album_filter {
                    if !song.album.to_lowercase().contains(&album.to_lowercase()) {
                        return false;
                    }
                }
                true
            })
            .collect();
        
        if filtered_songs.is_empty() {
            println!("No songs found matching the criteria.");
            return Ok(());
        }
        
        println!("Found {} songs:", filtered_songs.len());
        println!("{:-<80}", "");
        
        for song in filtered_songs {
            println!(
                "{} - {} [{}] ({})",
                song.title,
                song.artist,
                song.album,
                song.duration_formatted()
            );
        }
        
        Ok(())
    }

    fn description(&self) -> &'static str {
        "List all songs in the database with optional filtering"
    }
}
