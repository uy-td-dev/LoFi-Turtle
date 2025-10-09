use crate::commands::Command;
use crate::config::Config;
use crate::error::Result;
use crate::library::{Database, MusicScanner};
use async_trait::async_trait;

/// Command to scan music library and update database
pub struct ScanCommand {
    force_rescan: bool,
}

impl ScanCommand {
    pub fn new(force_rescan: bool) -> Self {
        Self { force_rescan }
    }
}

#[async_trait]
impl Command for ScanCommand {
    async fn execute(&self, config: &Config) -> Result<()> {
        log::info!("Scanning music library at: {}", config.music_dir.display());
        
        let database = Database::new(&config.database_path)?;
        let scanner = MusicScanner::new();
        
        println!("Scanning music directory: {}", config.music_dir.display());
        let songs = scanner.scan_directory(&config.music_dir)?;
        
        println!("Found {} songs. Updating database...", songs.len());
        
        if self.force_rescan {
            println!("Force rescan enabled - clearing existing database entries...");
            database.clear_all_songs()?;
        }
        
        let mut added_count = 0;
        let mut updated_count = 0;
        let mut error_count = 0;
        
        for song in &songs {
            match database.insert_or_update_song(song) {
                Ok(was_new) => {
                    if was_new {
                        added_count += 1;
                    } else {
                        updated_count += 1;
                    }
                }
                Err(e) => {
                    log::warn!("Failed to insert song {}: {}", song.path, e);
                    error_count += 1;
                }
            }
        }
        
        println!("Scan completed:");
        println!("  - Added: {} new songs", added_count);
        println!("  - Updated: {} existing songs", updated_count);
        if error_count > 0 {
            println!("  - Errors: {} songs failed to process", error_count);
        }
        
        Ok(())
    }

    fn description(&self) -> &'static str {
        "Scan music library and update database"
    }
}
