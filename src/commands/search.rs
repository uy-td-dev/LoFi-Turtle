use crate::commands::Command;
use crate::config::Config;
use crate::error::Result;
use crate::library::Database;

/// Command to search for songs in the database
pub struct SearchCommand {
    query: String,
}

impl SearchCommand {
    pub fn new(query: String) -> Self {
        Self { query }
    }
}

impl Command for SearchCommand {
    fn execute(&self, config: &Config) -> Result<()> {
        let database = Database::new(&config.database_path)?;
        let songs = database.get_all_songs()?;
        
        let query_lower = self.query.to_lowercase();
        let matching_songs: Vec<_> = songs
            .iter()
            .filter(|song| {
                song.title.to_lowercase().contains(&query_lower) ||
                song.artist.to_lowercase().contains(&query_lower) ||
                song.album.to_lowercase().contains(&query_lower)
            })
            .collect();
        
        if matching_songs.is_empty() {
            println!("No songs found matching '{}'", self.query);
            return Ok(());
        }
        
        println!("Found {} songs matching '{}':", matching_songs.len(), self.query);
        println!("{:-<80}", "");
        
        for song in matching_songs {
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
        "Search for songs by title, artist, or album"
    }
}
