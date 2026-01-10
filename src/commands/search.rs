use crate::commands::Command;
use crate::config::Config;
use crate::error::Result;
use crate::library::Database;

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
        let songs = database.search_songs(&self.query)?;

        if songs.is_empty() {
            println!("No songs found matching '{}'.", self.query);
        } else {
            println!("Found {} songs matching '{}':", songs.len(), self.query);
            println!("{:<50} | {:<30} | {:<30} | {:<10}", "Title", "Artist", "Album", "Duration");
            println!("{:-<50}-+-{:-<30}-+-{:-<30}-+-{:-<10}", "", "", "", "");

            for song in songs {
                println!("{:<50} | {:<30} | {:<30} | {:<10}",
                    truncate(&song.title, 50),
                    truncate(&song.artist, 30),
                    truncate(&song.album, 30),
                    song.duration_formatted()
                );
            }
        }

        Ok(())
    }

    fn description(&self) -> &'static str {
        "Search for songs in the library"
    }
}

fn truncate(s: &str, max_width: usize) -> String {
    if s.len() > max_width {
        format!("{}...", &s[0..max_width-3])
    } else {
        s.to_string()
    }
}
