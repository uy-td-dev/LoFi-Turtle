use crate::commands::Command;
use crate::config::Config;
use crate::error::Result;
use crate::library::{Database, MusicScanner};
use std::time::Instant;

pub struct ScanCommand {
    force: bool,
}

impl ScanCommand {
    pub fn new(force: bool) -> Self {
        Self { force }
    }
}

impl Command for ScanCommand {
    fn execute(&self, config: &Config) -> Result<()> {
        println!("Scanning music directory: {}", config.music_dir.display());
        let start = Instant::now();

        let mut database = Database::new(&config.database_path)?;

        if self.force {
            println!("Force scan enabled. Clearing existing database...");
            database.clear_all_songs()?;
        }

        let scanner = MusicScanner::new();
        let songs = scanner.scan_directory(&config.music_dir)?;

        println!("Found {} songs. Updating database...", songs.len());

        // Use bulk insert for better performance
        let count = database.insert_songs_bulk(&songs)?;

        let duration = start.elapsed();
        println!("Scan completed in {:.2?}. Added/Updated {} songs.", duration, count);

        Ok(())
    }

    fn description(&self) -> &'static str {
        "Scan music directory and update the library database"
    }
}
