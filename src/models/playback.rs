use serde::{Deserialize, Serialize};
use rand::{seq::SliceRandom, rng};
use std::collections::VecDeque;

/// Playback modes for the music player
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatMode {
    /// No repeat - play through the playlist once
    None,
    /// Repeat the current song
    Single,
    /// Repeat the entire playlist
    Playlist,
}

impl Default for RepeatMode {
    fn default() -> Self {
        Self::None
    }
}

impl RepeatMode {
}

/// Playback state for the music player
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaybackState {
    pub shuffle: bool,
    pub repeat_mode: RepeatMode,
    pub current_song_index: usize,
    pub is_playing: bool,
    pub is_paused: bool,
    pub volume: f32,
    /// Shuffle queue for fair randomization - stores indices of songs to play
    #[serde(skip)]
    pub shuffle_queue: VecDeque<usize>,
    /// Original playlist order for when shuffle is disabled
    #[serde(skip)]
    pub original_order: Vec<usize>,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            shuffle: false,
            repeat_mode: RepeatMode::default(),
            current_song_index: 0,
            is_playing: false,
            is_paused: false,
            volume: 0.7, // 70% volume by default
            shuffle_queue: VecDeque::new(),
            original_order: Vec::new(),
        }
    }
}

impl PlaybackState {
    /// Create a new playback state
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Play the music
    #[allow(dead_code)]
    pub fn play(&mut self) {
        self.is_playing = true;
        self.is_paused = false;
    }

    /// Pause the music
    #[allow(dead_code)]
    pub fn pause(&mut self) {
        self.is_playing = false;
        self.is_paused = true;
    }

    /// Stop the music
    #[allow(dead_code)]
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.is_paused = false;
    }

    /// Set the volume
    #[allow(dead_code)]
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Cycle through repeat modes
    #[allow(dead_code)]
    pub fn cycle_repeat_mode(&mut self) {
        self.repeat_mode = match self.repeat_mode {
            RepeatMode::None => RepeatMode::Single,
            RepeatMode::Single => RepeatMode::Playlist,
            RepeatMode::Playlist => RepeatMode::None,
        };
    }

    /// Toggle shuffle mode with fair randomization
    pub fn toggle_shuffle(&mut self, playlist_size: usize) {
        self.shuffle = !self.shuffle;
        
        if self.shuffle {
            self.enable_shuffle(playlist_size);
        } else {
            self.disable_shuffle();
        }
    }

    /// Enable shuffle mode with Fisher-Yates algorithm for fair randomization
    pub fn enable_shuffle(&mut self, playlist_size: usize) {
        if playlist_size == 0 {
            return;
        }

        // Store original order
        self.original_order = (0..playlist_size).collect();
        
        // Create shuffled queue using Fisher-Yates algorithm
        let mut indices: Vec<usize> = (0..playlist_size).collect();
        indices.shuffle(&mut rng());
        
        // Remove current song from shuffle queue and add it to front
        if let Some(pos) = indices.iter().position(|&x| x == self.current_song_index) {
            indices.remove(pos);
        }
        
        self.shuffle_queue = indices.into();
    }

    /// Disable shuffle mode and restore original order
    pub fn disable_shuffle(&mut self) {
        self.shuffle_queue.clear();
        self.original_order.clear();
    }


    /// Get the next song index based on current state with improved shuffle
    pub fn next_song_index(&mut self, playlist_size: usize) -> Option<usize> {
        if playlist_size == 0 {
            return None;
        }

        match self.repeat_mode {
            RepeatMode::Single => Some(self.current_song_index),
            RepeatMode::None | RepeatMode::Playlist => {
                let next_index = if self.shuffle {
                    // Use fair shuffle queue
                    if let Some(next) = self.shuffle_queue.pop_front() {
                        next
                    } else {
                        // Queue is empty, regenerate if repeat mode is Playlist
                        if self.repeat_mode == RepeatMode::Playlist {
                            self.enable_shuffle(playlist_size);
                            self.shuffle_queue.pop_front().unwrap_or(0)
                        } else {
                            return None; // End of shuffled playlist
                        }
                    }
                } else {
                    (self.current_song_index + 1) % playlist_size
                };

                if self.repeat_mode == RepeatMode::None && !self.shuffle && next_index == 0 && self.current_song_index == playlist_size - 1 {
                    None // End of playlist, no repeat
                } else {
                    Some(next_index)
                }
            }
        }
    }

    /// Get the previous song index based on current state
    pub fn previous_song_index(&mut self, playlist_size: usize) -> Option<usize> {
        if playlist_size == 0 {
            return None;
        }

        match self.repeat_mode {
            RepeatMode::Single => Some(self.current_song_index),
            RepeatMode::None | RepeatMode::Playlist => {
                let prev_index = if self.shuffle {
                    // For shuffle, we'll use a simple previous logic
                    // In a real implementation, you might want to maintain a history
                    if self.current_song_index == 0 {
                        playlist_size - 1
                    } else {
                        self.current_song_index - 1
                    }
                } else {
                    if self.current_song_index == 0 {
                        playlist_size - 1
                    } else {
                        self.current_song_index - 1
                    }
                };

                Some(prev_index)
            }
        }
    }

    /// Update current song index and manage shuffle queue
    pub fn set_current_song_index(&mut self, index: usize, playlist_size: usize) {
        self.current_song_index = index;
        
        // If shuffle is enabled and queue is empty, regenerate
        if self.shuffle && self.shuffle_queue.is_empty() && playlist_size > 0 {
            self.enable_shuffle(playlist_size);
        }
    }
}




