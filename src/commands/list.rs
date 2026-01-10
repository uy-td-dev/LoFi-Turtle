use crate::commands::Command;
use crate::config::Config;
use crate::error::Result;
use crate::library::Database;

pub struct ListCommand {
    artist: Option<String>,
    album: Option<String>,
}

impl ListCommand {
    pub fn new(artist: Option<String>, album: Option<String>) -> Self {
        Self { artist, album }
    }
}

impl Command for ListCommand {
    fn execute(&self, config: &Config) -> Result<()> {
        let database = Database::new(&config.database_path)?;
        let songs = database.get_all_songs()?;

        let filtered_songs: Vec<_> = songs.into_iter().filter(|song| {
            if let Some(ref artist) = self.artist {
                if !song.artist.to_lowercase().contains(&artist.to_lowercase()) {
                    return false;
                }
            }
            if let Some(ref album) = self.album {
                if !song.album.to_lowercase().contains(&album.to_lowercase()) {
                    return false;
                }
            }
            true
        }).collect();

        if filtered_songs.is_empty() {
            println!("No songs found matching criteria.");
        } else {
            println!("Found {} songs:", filtered_songs.len());
            println!("{:<50} | {:<30} | {:<30} | {:<10}", "Title", "Artist", "Album", "Duration");
            println!("{:-<50}-+-{:-<30}-+-{:-<30}-+-{:-<10}", "", "", "", "");

            for song in filtered_songs {
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
        "List songs in the library with optional filtering"
    }
}

fn truncate(s: &str, max_width: usize) -> String {
    if s.len() > max_width {
        format!("{}...", &s[0..max_width-3])
    } else {
        s.to_string()
    }
}
