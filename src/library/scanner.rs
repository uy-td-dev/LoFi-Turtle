use crate::models::Song;
use crate::error::{LofiTurtleError, Result};
use lofty::prelude::*;
use lofty::probe::Probe;
use std::fs;
use std::path::Path;

pub struct MusicScanner;

impl MusicScanner {
    pub fn new() -> Self {
        Self
    }

    /// Scan directory and return a list of songs
    /// This version collects all songs into a vector
    pub fn scan_directory<P: AsRef<Path>>(&self, dir_path: P) -> Result<Vec<Song>> {
        let mut songs = Vec::new();
        self.scan_recursive(dir_path.as_ref(), &mut songs)?;
        Ok(songs)
    }

    fn scan_recursive(&self, dir: &Path, songs: &mut Vec<Song>) -> Result<()> {
        let entries = fs::read_dir(dir).map_err(|e| LofiTurtleError::FileSystem(e))?;

        for entry in entries {
            let entry = entry.map_err(LofiTurtleError::FileSystem)?;
            let path = entry.path();

            if path.is_dir() {
                if let Err(e) = self.scan_recursive(&path, songs) {
                    log::warn!("Failed to scan directory {}: {}", path.display(), e);
                }
            } else if self.is_audio_file(&path) {
                match self.extract_metadata(&path) {
                    Ok(song) => songs.push(song),
                    Err(e) => log::warn!("Failed to extract metadata from {}: {}", path.display(), e),
                }
            }
        }

        Ok(())
    }

    fn is_audio_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            matches!(ext.as_str(), "mp3" | "flac" | "aac" | "m4a" | "ogg" | "wav")
        } else {
            false
        }
    }

    pub fn extract_metadata(&self, path: &Path) -> Result<Song> {
        let tagged_file = Probe::open(path)
            .map_err(|e| LofiTurtleError::UnsupportedFormat(format!("Failed to open audio file '{}': {}", path.display(), e)))?
            .read()
            .map_err(|e| LofiTurtleError::UnsupportedFormat(format!("Failed to read audio file '{}': {}", path.display(), e)))?;

        let properties = tagged_file.properties();
        let duration = properties.duration().as_secs();

        let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());

        let (title, artist, album) = if let Some(tag) = tag {
            let title = tag.title()
                .map(|t| t.to_string())
                .unwrap_or_else(|| self.extract_title_from_filename(path));
            
            let artist = tag.artist()
                .map(|a| a.to_string())
                .unwrap_or_else(|| "Unknown Artist".to_string());
            
            let album = tag.album()
                .map(|a| a.to_string())
                .unwrap_or_else(|| "Unknown Album".to_string());

            (title, artist, album)
        } else {
            (
                self.extract_title_from_filename(path),
                "Unknown Artist".to_string(),
                "Unknown Album".to_string(),
            )
        };

        Ok(Song::new(
            path.to_string_lossy().to_string(),
            title,
            artist,
            album,
            duration,
        ))
    }

    fn extract_title_from_filename(&self, path: &Path) -> String {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown Title")
            .to_string()
    }
}
