use crate::error::{LofiTurtleError, Result};
use crate::models::{Song, PlaybackState};
use rodio::{Decoder, OutputStreamBuilder, Sink};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Events that can be sent to control the gapless player
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PlayerEvent {
    Play,
    Pause,
    Stop,
    Next,
    Previous,
    Seek(Duration),
    SetVolume(f32),
    LoadPlaylist(Vec<Song>),
    ToggleShuffle,
    CycleRepeat,
}

/// Status updates from the player
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PlayerStatus {
    pub current_song: Option<Song>,
    pub position: Duration,
    pub duration: Duration,
    pub is_playing: bool,
    pub is_paused: bool,
    pub volume: f32,
    pub playback_state: PlaybackState,
}

/// Enhanced audio player with gapless playback support
#[allow(dead_code)]
pub struct GaplessPlayer {
    _stream: rodio::OutputStream,
    sink: Arc<Mutex<Sink>>,
    current_song: Arc<Mutex<Option<Song>>>,
    playlist: Arc<Mutex<Vec<Song>>>,
    playback_manager: Arc<Mutex<PlaybackState>>,
    event_sender: mpsc::UnboundedSender<PlayerEvent>,
    status_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<PlayerStatus>>>>,
    is_running: Arc<Mutex<bool>>,
}

impl GaplessPlayer {
    /// Create a new gapless player
    #[allow(dead_code)]
    pub fn new() -> Result<Self> {
        // Initialize rodio audio output stream using correct 0.21.1 API
        let stream_handle = OutputStreamBuilder::open_default_stream()
            .map_err(|e| LofiTurtleError::AudioPlayback(format!("Failed to open audio stream: {}", e)))?;
        let sink = Sink::connect_new(&stream_handle.mixer());

        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let (status_sender, status_receiver) = mpsc::unbounded_channel();

        let player = Self {
            _stream: stream_handle,
            sink: Arc::new(Mutex::new(sink)),
            current_song: Arc::new(Mutex::new(None)),
            playlist: Arc::new(Mutex::new(Vec::new())),
            playback_manager: Arc::new(Mutex::new(PlaybackState::new())),
            event_sender,
            status_receiver: Arc::new(Mutex::new(Some(status_receiver))),
            is_running: Arc::new(Mutex::new(true)),
        };

        // Start the player control loop
        player.start_control_loop(event_receiver, status_sender)?;

        Ok(player)
    }

    /// Start the player control loop in a separate thread
    #[allow(dead_code)]
    fn start_control_loop(
        &self,
        mut event_receiver: mpsc::UnboundedReceiver<PlayerEvent>,
        status_sender: mpsc::UnboundedSender<PlayerStatus>,
    ) -> Result<()> {
        let sink = Arc::clone(&self.sink);
        let current_song = Arc::clone(&self.current_song);
        let playlist = Arc::clone(&self.playlist);
        let playback_manager = Arc::clone(&self.playback_manager);
        let is_running = Arc::clone(&self.is_running);

        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut last_status_update = Instant::now();
                let status_update_interval = Duration::from_millis(100);

                while *is_running.lock().unwrap() {
                    // Handle events
                    while let Ok(event) = event_receiver.try_recv() {
                        if let Err(e) = Self::handle_event(
                            &event,
                            &sink,
                            &current_song,
                            &playlist,
                            &playback_manager,
                        ) {
                            eprintln!("Error handling player event: {}", e);
                        }
                    }

                    // Send status updates periodically
                    if last_status_update.elapsed() >= status_update_interval {
                        let status = Self::get_current_status(
                            &sink,
                            &current_song,
                            &playback_manager,
                        );
                        
                        if status_sender.send(status).is_err() {
                            break; // Receiver dropped
                        }
                        
                        last_status_update = Instant::now();
                    }

                    // Check if current song ended and handle gapless transition
                    {
                        let sink_guard = sink.lock().unwrap();
                        if sink_guard.empty() {
                            drop(sink_guard);
                            
                            if let Err(e) = Self::handle_song_end(
                                &sink,
                                &current_song,
                                &playlist,
                                &playback_manager,
                            ) {
                                eprintln!("Error handling song end: {}", e);
                            }
                        }
                    }

                    // Small delay to prevent busy waiting
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            });
        });

        Ok(())
    }

    /// Handle player events
    #[allow(dead_code)]
    fn handle_event(
        event: &PlayerEvent,
        sink: &Arc<Mutex<Sink>>,
        current_song: &Arc<Mutex<Option<Song>>>,
        playlist: &Arc<Mutex<Vec<Song>>>,
        playback_manager: &Arc<Mutex<PlaybackState>>,
    ) -> Result<()> {
        match event {
            PlayerEvent::Play => {
                let mut manager = playback_manager.lock().unwrap();
                manager.play();
                
                let sink_guard = sink.lock().unwrap();
                sink_guard.play();
            }
            PlayerEvent::Pause => {
                let mut manager = playback_manager.lock().unwrap();
                manager.pause();
                
                let sink_guard = sink.lock().unwrap();
                sink_guard.pause();
            }
            PlayerEvent::Stop => {
                let mut manager = playback_manager.lock().unwrap();
                manager.stop();
                
                let sink_guard = sink.lock().unwrap();
                sink_guard.stop();
                
                *current_song.lock().unwrap() = None;
            }
            PlayerEvent::Next => {
                Self::play_next_song(sink, current_song, playlist, playback_manager)?;
            }
            PlayerEvent::Previous => {
                Self::play_previous_song(sink, current_song, playlist, playback_manager)?;
            }
            PlayerEvent::Seek(position) => {
                // Note: Rodio doesn't support seeking directly
                // This would require a more advanced audio library like symphonia
                println!("Seeking to {:?} (not implemented with current audio backend)", position);
            }
            PlayerEvent::SetVolume(volume) => {
                let mut manager = playback_manager.lock().unwrap();
                manager.set_volume(*volume);
                
                let sink_guard = sink.lock().unwrap();
                sink_guard.set_volume(*volume);
            }
            PlayerEvent::LoadPlaylist(songs) => {
                *playlist.lock().unwrap() = songs.clone();
                
                let mut manager = playback_manager.lock().unwrap();
                // Playlist size is handled implicitly
                
                // Start playing the first song if playlist is not empty
                if !songs.is_empty() {
                    manager.current_song_index = 0;
                    drop(manager);
                    Self::load_and_play_current_song(sink, current_song, playlist, playback_manager)?;
                }
            }
            PlayerEvent::ToggleShuffle => {
                let playlist_guard = playlist.lock().unwrap();
                let playlist_size = playlist_guard.len();
                drop(playlist_guard);
                
                let mut manager = playback_manager.lock().unwrap();
                manager.toggle_shuffle(playlist_size);
            }
            PlayerEvent::CycleRepeat => {
                let mut manager = playback_manager.lock().unwrap();
                manager.cycle_repeat_mode();
            }
        }
        
        Ok(())
    }

    /// Get current player status
    #[allow(dead_code)]
    fn get_current_status(
        sink: &Arc<Mutex<Sink>>,
        current_song: &Arc<Mutex<Option<Song>>>,
        playback_manager: &Arc<Mutex<PlaybackState>>,
    ) -> PlayerStatus {
        let sink_guard = sink.lock().unwrap();
        let song_guard = current_song.lock().unwrap();
        let manager_guard = playback_manager.lock().unwrap();
        
        PlayerStatus {
            current_song: song_guard.clone(),
            position: Duration::from_secs(0), // Would need more advanced audio library for accurate position
            duration: song_guard.as_ref().map(|s| Duration::from_secs(s.duration)).unwrap_or_default(),
            is_playing: manager_guard.is_playing && !sink_guard.is_paused(),
            is_paused: manager_guard.is_paused,
            volume: manager_guard.volume,
            playback_state: manager_guard.clone(),
        }
    }

    /// Handle when a song ends (for gapless playback)
    #[allow(dead_code)]
    fn handle_song_end(
        sink: &Arc<Mutex<Sink>>,
        current_song: &Arc<Mutex<Option<Song>>>,
        playlist: &Arc<Mutex<Vec<Song>>>,
        playback_manager: &Arc<Mutex<PlaybackState>>,
    ) -> Result<()> {
        let mut manager = playback_manager.lock().unwrap();
        let playlist_guard = playlist.lock().unwrap();
        let playlist_size = playlist_guard.len();
        
        if let Some(next_index) = manager.next_song_index(playlist_size) {
            drop(manager);
            drop(playlist_guard);
            
            // Update current song index and play next song
            {
                let mut manager = playback_manager.lock().unwrap();
                manager.current_song_index = next_index;
            }
            
            Self::load_and_play_current_song(sink, current_song, playlist, playback_manager)?;
        } else {
            // End of playlist reached
            let mut manager = playback_manager.lock().unwrap();
            manager.stop();
            drop(manager);
            
            *current_song.lock().unwrap() = None;
        }
        
        Ok(())
    }

    /// Play the next song in the playlist
    #[allow(dead_code)]
    fn play_next_song(
        sink: &Arc<Mutex<Sink>>,
        current_song: &Arc<Mutex<Option<Song>>>,
        playlist: &Arc<Mutex<Vec<Song>>>,
        playback_manager: &Arc<Mutex<PlaybackState>>,
    ) -> Result<()> {
        let mut manager = playback_manager.lock().unwrap();
        let playlist_guard = playlist.lock().unwrap();
        let playlist_size = playlist_guard.len();
        
        if let Some(next_index) = manager.next_song_index(playlist_size) {
            drop(manager);
            drop(playlist_guard);
            
            {
                let mut manager = playback_manager.lock().unwrap();
                manager.current_song_index = next_index;
            }
            
            Self::load_and_play_current_song(sink, current_song, playlist, playback_manager)?;
        }
        
        Ok(())
    }

    /// Play the previous song in the playlist
    #[allow(dead_code)]
    fn play_previous_song(
        sink: &Arc<Mutex<Sink>>,
        current_song: &Arc<Mutex<Option<Song>>>,
        playlist: &Arc<Mutex<Vec<Song>>>,
        playback_manager: &Arc<Mutex<PlaybackState>>,
    ) -> Result<()> {
        let mut manager = playback_manager.lock().unwrap();
        let playlist_guard = playlist.lock().unwrap();
        let playlist_size = playlist_guard.len();
        
        if let Some(prev_index) = manager.previous_song_index(playlist_size) {
            drop(manager);
            drop(playlist_guard);
            
            {
                let mut manager = playback_manager.lock().unwrap();
                manager.current_song_index = prev_index;
            }
            
            Self::load_and_play_current_song(sink, current_song, playlist, playback_manager)?;
        }
        
        Ok(())
    }

    /// Load and play the current song based on the current index
    #[allow(dead_code)]
    fn load_and_play_current_song(
        sink: &Arc<Mutex<Sink>>,
        current_song: &Arc<Mutex<Option<Song>>>,
        playlist: &Arc<Mutex<Vec<Song>>>,
        playback_manager: &Arc<Mutex<PlaybackState>>,
    ) -> Result<()> {
        let manager = playback_manager.lock().unwrap();
        let current_index = manager.current_song_index;
        let playlist_guard = playlist.lock().unwrap();
        
        if let Some(song) = playlist_guard.get(current_index) {
            let song_clone = song.clone();
            drop(manager);
            drop(playlist_guard);
            
            // Load the audio file
            let file = File::open(&song_clone.path)
                .map_err(|e| LofiTurtleError::AudioPlayback(format!("Failed to open audio file: {}", e)))?;
            
            let source = Decoder::new(BufReader::new(file))
                .map_err(|e| LofiTurtleError::AudioPlayback(format!("Failed to decode audio: {}", e)))?;
            
            // Stop current playback and clear the sink
            {
                let sink_guard = sink.lock().unwrap();
                sink_guard.stop();
                // Note: Rodio doesn't have a clear method, so we create a new sink
            }
            
            // Add the new source to the sink for gapless playback
            {
                let sink_guard = sink.lock().unwrap();
                sink_guard.append(source);
                sink_guard.play();
            }
            
            // Update current song
            *current_song.lock().unwrap() = Some(song_clone);
            
            // Update playback state
            {
                let mut manager = playback_manager.lock().unwrap();
                manager.play();
            }
        }
        
        Ok(())
    }

    /// Send an event to the player
    #[allow(dead_code)]
    pub fn send_event(&self, event: PlayerEvent) -> Result<()> {
        self.event_sender.send(event)
            .map_err(|e| LofiTurtleError::AudioPlayback(format!("Failed to send player event: {}", e)))?;
        Ok(())
    }

    /// Get the status receiver (should be called only once)
    #[allow(dead_code)]
    pub fn take_status_receiver(&self) -> Option<mpsc::UnboundedReceiver<PlayerStatus>> {
        self.status_receiver.lock().unwrap().take()
    }

    /// Load a playlist and start playing
    #[allow(dead_code)]
    pub fn load_playlist(&self, songs: Vec<Song>) -> Result<()> {
        self.send_event(PlayerEvent::LoadPlaylist(songs))
    }

    /// Play the current song or resume if paused
    #[allow(dead_code)]
    pub fn play(&self) -> Result<()> {
        self.send_event(PlayerEvent::Play)
    }

    /// Pause playback
    #[allow(dead_code)]
    pub fn pause(&self) -> Result<()> {
        self.send_event(PlayerEvent::Pause)
    }

    /// Stop playback
    #[allow(dead_code)]
    pub fn stop(&self) -> Result<()> {
        self.send_event(PlayerEvent::Stop)
    }

    /// Skip to next song
    #[allow(dead_code)]
    pub fn next(&self) -> Result<()> {
        self.send_event(PlayerEvent::Next)
    }

    /// Go to previous song
    #[allow(dead_code)]
    pub fn previous(&self) -> Result<()> {
        self.send_event(PlayerEvent::Previous)
    }

    /// Set volume (0.0 to 1.0)
    #[allow(dead_code)]
    pub fn set_volume(&self, volume: f32) -> Result<()> {
        self.send_event(PlayerEvent::SetVolume(volume.clamp(0.0, 1.0)))
    }

    /// Toggle shuffle mode
    #[allow(dead_code)]
    pub fn toggle_shuffle(&self) -> Result<()> {
        self.send_event(PlayerEvent::ToggleShuffle)
    }

    /// Cycle through repeat modes
    #[allow(dead_code)]
    pub fn cycle_repeat(&self) -> Result<()> {
        self.send_event(PlayerEvent::CycleRepeat)
    }

    /// Get current playback manager state (for UI display)
    #[allow(dead_code)]
    pub fn get_playback_state(&self) -> PlaybackState {
        self.playback_manager.lock().unwrap().clone()
    }

    /// Get current song
    #[allow(dead_code)]
    pub fn get_current_song(&self) -> Option<Song> {
        self.current_song.lock().unwrap().clone()
    }

    /// Get current playlist
    #[allow(dead_code)]
    pub fn get_playlist(&self) -> Vec<Song> {
        self.playlist.lock().unwrap().clone()
    }
}

impl Drop for GaplessPlayer {
    fn drop(&mut self) {
        *self.is_running.lock().unwrap() = false;
    }
}

