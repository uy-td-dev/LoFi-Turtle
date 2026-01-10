//! Core playback engine
//! 
//! Handles the core music playback logic, state management, and audio control.

use crate::error::{LofiTurtleError, Result};
use crate::models::{Song, PlaybackState};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Core playback engine for music player
pub struct PlaybackEngine {
    /// Current playback state
    state: Arc<Mutex<PlaybackState>>,
    
    /// Current song being played
    current_song: Option<Song>,
    
    /// Playback position in seconds
    position: Duration,
    
    /// Total duration of current song
    duration: Option<Duration>,
    
    /// Volume level (0.0 to 1.0)
    volume: f32,
    
    /// Whether shuffle mode is enabled
    shuffle: bool,
    
    /// Repeat mode
    repeat_mode: RepeatMode,
}

/// Repeat mode options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RepeatMode {
    None,
    Single,
    Playlist,
}

impl PlaybackEngine {
    /// Create a new playback engine
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(PlaybackState::Stopped)),
            current_song: None,
            position: Duration::from_secs(0),
            duration: None,
            volume: 0.7,
            shuffle: false,
            repeat_mode: RepeatMode::None,
        }
    }
    
    /// Start playing a song
    pub fn play(&mut self, song: Song) -> Result<()> {
        self.current_song = Some(song);
        self.position = Duration::from_secs(0);
        
        if let Ok(mut state) = self.state.lock() {
            *state = PlaybackState::Playing;
        }
        
        Ok(())
    }
    
    /// Pause playback
    pub fn pause(&mut self) -> Result<()> {
        if let Ok(mut state) = self.state.lock() {
            *state = PlaybackState::Paused;
        }
        Ok(())
    }
    
    /// Resume playback
    pub fn resume(&mut self) -> Result<()> {
        if let Ok(mut state) = self.state.lock() {
            *state = PlaybackState::Playing;
        }
        Ok(())
    }
    
    /// Stop playback
    pub fn stop(&mut self) -> Result<()> {
        if let Ok(mut state) = self.state.lock() {
            *state = PlaybackState::Stopped;
        }
        self.position = Duration::from_secs(0);
        Ok(())
    }
    
    /// Toggle play/pause
    pub fn toggle_playback(&mut self) -> Result<()> {
        if let Ok(state) = self.state.lock() {
            match *state {
                PlaybackState::Playing => self.pause(),
                PlaybackState::Paused => self.resume(),
                PlaybackState::Stopped => {
                    if let Some(ref song) = self.current_song {
                        self.play(song.clone())
                    } else {
                        Ok(())
                    }
                }
            }
        } else {
            Err(LofiTurtleError::AudioPlayback("Failed to access playback state".to_string()))
        }
    }
    
    /// Set volume (0.0 to 1.0)
    pub fn set_volume(&mut self, volume: f32) -> Result<()> {
        self.volume = volume.clamp(0.0, 1.0);
        Ok(())
    }
    
    /// Adjust volume by delta
    pub fn adjust_volume(&mut self, delta: f32) -> Result<()> {
        self.set_volume(self.volume + delta)
    }
    
    /// Seek to position
    pub fn seek(&mut self, position: Duration) -> Result<()> {
        if let Some(duration) = self.duration {
            if position <= duration {
                self.position = position;
                return Ok(());
            }
        }
        self.position = position;
        Ok(())
    }
    
    /// Get current playback state
    pub fn get_state(&self) -> PlaybackState {
        self.state.lock().unwrap_or_else(|_| PlaybackState::Stopped).clone()
    }
    
    /// Get current song
    pub fn get_current_song(&self) -> Option<&Song> {
        self.current_song.as_ref()
    }
    
    /// Get current position
    pub fn get_position(&self) -> Duration {
        self.position
    }
    
    /// Get total duration
    pub fn get_duration(&self) -> Option<Duration> {
        self.duration
    }
    
    /// Get volume
    pub fn get_volume(&self) -> f32 {
        self.volume
    }
    
    /// Get shuffle mode
    pub fn is_shuffle(&self) -> bool {
        self.shuffle
    }
    
    /// Set shuffle mode
    pub fn set_shuffle(&mut self, shuffle: bool) {
        self.shuffle = shuffle;
    }
    
    /// Get repeat mode
    pub fn get_repeat_mode(&self) -> RepeatMode {
        self.repeat_mode
    }
    
    /// Set repeat mode
    pub fn set_repeat_mode(&mut self, mode: RepeatMode) {
        self.repeat_mode = mode;
    }
    
    /// Update playback position (called by audio backend)
    pub fn update_position(&mut self, position: Duration) {
        self.position = position;
    }
    
    /// Set total duration (called when song is loaded)
    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = Some(duration);
    }
    
    /// Check if playback has ended
    pub fn is_ended(&self) -> bool {
        if let Some(duration) = self.duration {
            self.position >= duration
        } else {
            false
        }
    }
    
    /// Get progress as percentage (0.0 to 1.0)
    pub fn get_progress(&self) -> f32 {
        if let Some(duration) = self.duration {
            if duration.as_secs() > 0 {
                (self.position.as_secs_f32() / duration.as_secs_f32()).min(1.0)
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
}

impl Default for PlaybackEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Song;
    
    fn create_test_song() -> Song {
        Song {
            id: 1,
            title: "Test Song".to_string(),
            artist: Some("Test Artist".to_string()),
            album: Some("Test Album".to_string()),
            file_path: "/test/path.mp3".to_string(),
            duration: Some(180),
            track_number: Some(1),
            year: Some(2023),
            genre: Some("Test".to_string()),
            album_art_path: None,
        }
    }
    
    #[test]
    fn test_playback_engine_creation() {
        let engine = PlaybackEngine::new();
        assert_eq!(engine.get_state(), PlaybackState::Stopped);
        assert_eq!(engine.get_volume(), 0.7);
        assert!(!engine.is_shuffle());
        assert_eq!(engine.get_repeat_mode(), RepeatMode::None);
    }
    
    #[test]
    fn test_play_pause_resume() {
        let mut engine = PlaybackEngine::new();
        let song = create_test_song();
        
        // Play song
        engine.play(song).unwrap();
        assert_eq!(engine.get_state(), PlaybackState::Playing);
        
        // Pause
        engine.pause().unwrap();
        assert_eq!(engine.get_state(), PlaybackState::Paused);
        
        // Resume
        engine.resume().unwrap();
        assert_eq!(engine.get_state(), PlaybackState::Playing);
        
        // Stop
        engine.stop().unwrap();
        assert_eq!(engine.get_state(), PlaybackState::Stopped);
    }
    
    #[test]
    fn test_volume_control() {
        let mut engine = PlaybackEngine::new();
        
        // Set volume
        engine.set_volume(0.5).unwrap();
        assert_eq!(engine.get_volume(), 0.5);
        
        // Adjust volume
        engine.adjust_volume(0.2).unwrap();
        assert_eq!(engine.get_volume(), 0.7);
        
        // Test clamping
        engine.set_volume(1.5).unwrap();
        assert_eq!(engine.get_volume(), 1.0);
        
        engine.set_volume(-0.5).unwrap();
        assert_eq!(engine.get_volume(), 0.0);
    }
    
    #[test]
    fn test_progress_calculation() {
        let mut engine = PlaybackEngine::new();
        
        // Set duration
        engine.set_duration(Duration::from_secs(100));
        
        // Test progress at different positions
        engine.update_position(Duration::from_secs(0));
        assert_eq!(engine.get_progress(), 0.0);
        
        engine.update_position(Duration::from_secs(50));
        assert_eq!(engine.get_progress(), 0.5);
        
        engine.update_position(Duration::from_secs(100));
        assert_eq!(engine.get_progress(), 1.0);
    }
}
