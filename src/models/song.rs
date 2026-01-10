use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub id: String,        // MD5 hash of the file path
    pub path: String,      // Full path to the audio file
    pub title: String,     // Song title
    pub artist: String,    // Artist name
    pub album: String,     // Album name
    pub duration: u64,     // Duration in seconds
    
    // Performance optimization: Cache frequently accessed strings
    #[serde(skip)]
    display_name_cache: OnceLock<String>,
    #[serde(skip)]
    duration_formatted_cache: OnceLock<String>,
    #[serde(skip)]
    search_string_cache: OnceLock<String>,
}

impl Song {
    pub fn new(
        path: String,
        title: String,
        artist: String,
        album: String,
        duration: u64,
    ) -> Self {
        let id = format!("{:x}", md5::compute(&path));
        Self {
            id,
            path,
            title,
            artist,
            album,
            duration,
            display_name_cache: OnceLock::new(),
            duration_formatted_cache: OnceLock::new(),
            search_string_cache: OnceLock::new(),
        }
    }

    /// Performance optimized: Cache duration string to avoid repeated formatting
    pub fn duration_formatted(&self) -> &str {
        self.duration_formatted_cache.get_or_init(|| {
            let minutes = self.duration / 60;
            let seconds = self.duration % 60;
            format!("{:02}:{:02}", minutes, seconds)
        })
    }

    /// Performance optimized: Cache display name to avoid repeated string allocation
    pub fn display_name(&self) -> &str {
        self.display_name_cache.get_or_init(|| {
            if self.artist.is_empty() {
                self.title.clone()
            } else {
                format!("{} - {}", self.title, self.artist)
            }
        })
    }

    /// Check if the song matches the given query (case-insensitive)
    /// Uses a cached lowercased search string to avoid repeated allocations
    pub fn matches(&self, query_lower: &str) -> bool {
        let search_string = self.search_string_cache.get_or_init(|| {
            format!("{} {} {}", self.title, self.artist, self.album).to_lowercase()
        });
        search_string.contains(query_lower)
    }
}
