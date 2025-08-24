use crate::models::{Song, Playlist};
use crate::error::{LofiTurtleError, Result};
use rusqlite::{params, Connection};
use std::path::Path;
use chrono::{DateTime, Utc};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)
            .map_err(LofiTurtleError::Database)?;
        
        let db = Self { conn };
        db.create_tables()?;
        Ok(db)
    }

    fn create_tables(&self) -> Result<()> {
        // Create songs table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS songs (
                id TEXT PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                artist TEXT NOT NULL,
                album TEXT NOT NULL,
                duration INTEGER NOT NULL
            )",
            [],
        ).map_err(LofiTurtleError::Database)?;

        // Create playlists table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS playlists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        ).map_err(LofiTurtleError::Database)?;

        // Create playlist_songs junction table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS playlist_songs (
                playlist_id TEXT NOT NULL,
                song_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                PRIMARY KEY (playlist_id, song_id),
                FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
                FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE
            )",
            [],
        ).map_err(LofiTurtleError::Database)?;

        Ok(())
    }

    pub fn insert_song(&self, song: &Song) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO songs (id, path, title, artist, album, duration)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                song.id,
                song.path,
                song.title,
                song.artist,
                song.album,
                song.duration as i64
            ],
        ).map_err(LofiTurtleError::Database)?;

        Ok(())
    }

    pub fn get_all_songs(&self) -> Result<Vec<Song>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, title, artist, album, duration FROM songs ORDER BY title"
        ).map_err(LofiTurtleError::Database)?;

        let song_iter = stmt.query_map([], |row| {
            Ok(Song::new(
                row.get(1)?, // path
                row.get(2)?, // title
                row.get(3)?, // artist
                row.get(4)?, // album
                row.get::<_, i64>(5)? as u64, // duration
            ))
        }).map_err(LofiTurtleError::Database)?;

        let mut songs = Vec::new();
        for song in song_iter {
            songs.push(song.map_err(LofiTurtleError::Database)?);
        }

        Ok(songs)
    }

    #[allow(dead_code)] // Future feature: database search
    pub fn search_songs(&self, query: &str) -> Result<Vec<Song>> {
        let search_pattern = format!("%{}%", query.to_lowercase());
        
        let mut stmt = self.conn.prepare(
            "SELECT id, path, title, artist, album, duration FROM songs 
             WHERE LOWER(title) LIKE ?1 OR LOWER(artist) LIKE ?1 
             ORDER BY title"
        ).map_err(LofiTurtleError::Database)?;

        let song_iter = stmt.query_map([&search_pattern], |row| {
            Ok(Song::new(
                row.get(1)?, // path
                row.get(2)?, // title
                row.get(3)?, // artist
                row.get(4)?, // album
                row.get::<_, i64>(5)? as u64, // duration
            ))
        }).map_err(LofiTurtleError::Database)?;

        let mut songs = Vec::new();
        for song in song_iter {
            songs.push(song.map_err(LofiTurtleError::Database)?);
        }

        Ok(songs)
    }

    pub fn song_exists(&self, path: &str) -> Result<bool> {
        let mut stmt = self.conn.prepare("SELECT 1 FROM songs WHERE path = ?1")
            .map_err(LofiTurtleError::Database)?;
        
        let exists = stmt.exists([path])
            .map_err(LofiTurtleError::Database)?;
        
        Ok(exists)
    }

    /// Insert or update a song, returning true if it was newly inserted
    pub fn insert_or_update_song(&self, song: &Song) -> Result<bool> {
        let was_new = !self.song_exists(&song.path)?;
        self.insert_song(song)?;
        Ok(was_new)
    }

    /// Clear all songs from the database
    pub fn clear_all_songs(&self) -> Result<()> {
        self.conn.execute("DELETE FROM songs", [])
            .map_err(LofiTurtleError::Database)?;
        Ok(())
    }

    // Playlist management methods

    /// Create a new playlist
    pub fn create_playlist(&self, playlist: &Playlist) -> Result<()> {
        let created_at = playlist.created_at.to_rfc3339();
        let updated_at = playlist.updated_at.to_rfc3339();

        self.conn.execute(
            "INSERT INTO playlists (id, name, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                playlist.id,
                playlist.name,
                playlist.description,
                created_at,
                updated_at
            ],
        ).map_err(LofiTurtleError::Database)?;

        // Add songs to the playlist
        for (position, song_id) in playlist.song_ids.iter().enumerate() {
            self.add_song_to_playlist(&playlist.id, song_id, position)?;
        }

        Ok(())
    }

    /// Get all playlists
    pub fn get_all_playlists(&self) -> Result<Vec<Playlist>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, created_at, updated_at FROM playlists ORDER BY name"
        ).map_err(LofiTurtleError::Database)?;

        let playlist_iter = stmt.query_map([], |row| {
            let created_at_str: String = row.get(3)?;
            let updated_at_str: String = row.get(4)?;
            
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(3, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
            
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(4, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);

            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                song_ids: Vec::new(), // Will be populated separately
                created_at,
                updated_at,
            })
        }).map_err(LofiTurtleError::Database)?;

        let mut playlists = Vec::new();
        for playlist_result in playlist_iter {
            let mut playlist = playlist_result.map_err(LofiTurtleError::Database)?;
            playlist.song_ids = self.get_playlist_song_ids(&playlist.id)?;
            playlists.push(playlist);
        }

        Ok(playlists)
    }


    /// Get a playlist by name
    pub fn get_playlist_by_name(&self, name: &str) -> Result<Option<Playlist>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, created_at, updated_at FROM playlists WHERE name = ?1"
        ).map_err(LofiTurtleError::Database)?;

        let mut playlist_iter = stmt.query_map([name], |row| {
            let created_at_str: String = row.get(3)?;
            let updated_at_str: String = row.get(4)?;
            
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(3, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
            
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(4, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);

            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                song_ids: Vec::new(), // Will be populated separately
                created_at,
                updated_at,
            })
        }).map_err(LofiTurtleError::Database)?;

        if let Some(playlist_result) = playlist_iter.next() {
            let mut playlist = playlist_result.map_err(LofiTurtleError::Database)?;
            playlist.song_ids = self.get_playlist_song_ids(&playlist.id)?;
            Ok(Some(playlist))
        } else {
            Ok(None)
        }
    }

    /// Delete a playlist
    pub fn delete_playlist(&self, playlist_id: &str) -> Result<bool> {
        let rows_affected = self.conn.execute(
            "DELETE FROM playlists WHERE id = ?1",
            [playlist_id],
        ).map_err(LofiTurtleError::Database)?;

        Ok(rows_affected > 0)
    }

    /// Add a song to a playlist
    pub fn add_song_to_playlist(&self, playlist_id: &str, song_id: &str, position: usize) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO playlist_songs (playlist_id, song_id, position)
             VALUES (?1, ?2, ?3)",
            params![playlist_id, song_id, position as i64],
        ).map_err(LofiTurtleError::Database)?;

        // Update playlist's updated_at timestamp
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE playlists SET updated_at = ?1 WHERE id = ?2",
            params![now, playlist_id],
        ).map_err(LofiTurtleError::Database)?;

        Ok(())
    }

    /// Remove a song from a playlist
    pub fn remove_song_from_playlist(&self, playlist_id: &str, song_id: &str) -> Result<bool> {
        let rows_affected = self.conn.execute(
            "DELETE FROM playlist_songs WHERE playlist_id = ?1 AND song_id = ?2",
            params![playlist_id, song_id],
        ).map_err(LofiTurtleError::Database)?;

        if rows_affected > 0 {
            // Update playlist's updated_at timestamp
            let now = Utc::now().to_rfc3339();
            self.conn.execute(
                "UPDATE playlists SET updated_at = ?1 WHERE id = ?2",
                params![now, playlist_id],
            ).map_err(LofiTurtleError::Database)?;
        }

        Ok(rows_affected > 0)
    }

    /// Get song IDs for a playlist in order
    fn get_playlist_song_ids(&self, playlist_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT song_id FROM playlist_songs WHERE playlist_id = ?1 ORDER BY position"
        ).map_err(LofiTurtleError::Database)?;

        let song_id_iter = stmt.query_map([playlist_id], |row| {
            Ok(row.get::<_, String>(0)?)
        }).map_err(LofiTurtleError::Database)?;

        let mut song_ids = Vec::new();
        for song_id_result in song_id_iter {
            song_ids.push(song_id_result.map_err(LofiTurtleError::Database)?);
        }

        Ok(song_ids)
    }

    /// Get songs for a playlist
    pub fn get_playlist_songs(&self, playlist_id: &str) -> Result<Vec<Song>> {
        
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.path, s.title, s.artist, s.album, s.duration 
             FROM songs s
             JOIN playlist_songs ps ON s.id = ps.song_id
             WHERE ps.playlist_id = ?1
             ORDER BY ps.position"
        ).map_err(LofiTurtleError::Database)?;

        let song_iter = stmt.query_map([playlist_id], |row| {
            Ok(Song::new(
                row.get(1)?, // path
                row.get(2)?, // title
                row.get(3)?, // artist
                row.get(4)?, // album
                row.get::<_, i64>(5)? as u64, // duration
            ))
        }).map_err(LofiTurtleError::Database)?;

        let mut songs = Vec::new();
        for song_result in song_iter {
            songs.push(song_result.map_err(LofiTurtleError::Database)?);
        }

        Ok(songs)
    }

    /// Check if a playlist exists
    pub fn playlist_exists(&self, name: &str) -> Result<bool> {
        let mut stmt = self.conn.prepare("SELECT 1 FROM playlists WHERE name = ?1")
            .map_err(LofiTurtleError::Database)?;
        
        let exists = stmt.exists([name])
            .map_err(LofiTurtleError::Database)?;
        
        Ok(exists)
    }
}
