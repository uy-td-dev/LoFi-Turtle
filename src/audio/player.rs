use crate::error::{LofiTurtleError, Result};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum PlayerCommand {
    Play(String),  // Play song at given path
    Pause,
    Resume,
    Stop,
    #[allow(dead_code)] // Future feature: seeking
    Seek(u64),     // Seek to position in seconds
    #[allow(dead_code)] // Future feature: volume control
    SetVolume(f32), // Set volume (0.0 to 1.0)
    #[allow(dead_code)] // Used in audio thread communication
    SetShuffle(bool), // Enable/disable shuffle mode
    #[allow(dead_code)] // Used in audio thread communication
    SetRepeat(crate::models::RepeatMode), // Set repeat mode
    Quit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaybackStatus {
    pub state: PlayerState,
    pub current_position: u64,  // Current position in seconds
    pub total_duration: u64,    // Total duration in seconds
    pub current_song: Option<String>, // Path to current song
    pub volume: f32,
}

impl Default for PlaybackStatus {
    fn default() -> Self {
        Self {
            state: PlayerState::Stopped,
            current_position: 0,
            total_duration: 0,
            current_song: None,
            volume: 0.7,
        }
    }
}

pub struct AudioPlayer {
    command_sender: Sender<PlayerCommand>,
    status: Arc<Mutex<PlaybackStatus>>,
}

impl AudioPlayer {
    pub fn new() -> Result<Self> {
        let (command_sender, command_receiver) = mpsc::channel();
        let status = Arc::new(Mutex::new(PlaybackStatus::default()));
        let status_clone = Arc::clone(&status);

        // Spawn the audio thread
        thread::spawn(move || {
            if let Err(e) = Self::audio_thread(command_receiver, status_clone) {
                eprintln!("Audio thread error: {}", e);
            }
        });

        Ok(Self {
            command_sender,
            status,
        })
    }

    pub fn send_command(&self, command: PlayerCommand) -> Result<()> {
        self.command_sender
            .send(command)
            .map_err(|e| LofiTurtleError::ChannelError(format!("Failed to send command to audio thread: {}", e)))?;
        Ok(())
    }

    pub fn get_status(&self) -> PlaybackStatus {
        self.status.lock().unwrap().clone()
    }

    fn audio_thread(
        command_receiver: Receiver<PlayerCommand>,
        status: Arc<Mutex<PlaybackStatus>>,
    ) -> Result<()> {
        let stream_handle = OutputStreamBuilder::open_default_stream()
            .map_err(|e| LofiTurtleError::AudioPlayback(format!("Failed to create audio output stream: {}", e)))?;

        let mut sink: Option<Sink> = None;
        let mut playback_start_time: Option<Instant> = None;
        let mut paused_position: u64 = 0;

        loop {
            // Handle commands
            while let Ok(command) = command_receiver.try_recv() {
                match command {
                    PlayerCommand::Play(path) => {
                        // Stop current playback
                        if let Some(s) = sink.take() {
                            s.stop();
                        }

                        match Self::load_audio_file(&path, &stream_handle) {
                            Ok((new_sink, duration)) => {
                                sink = Some(new_sink);
                                playback_start_time = Some(Instant::now());
                                paused_position = 0;

                                let mut status_guard = status.lock().unwrap();
                                status_guard.state = PlayerState::Playing;
                                status_guard.current_song = Some(path);
                                status_guard.total_duration = duration;
                                status_guard.current_position = 0;
                            }
                            Err(e) => {
                                eprintln!("Failed to load audio file: {}", e);
                                let mut status_guard = status.lock().unwrap();
                                status_guard.state = PlayerState::Stopped;
                            }
                        }
                    }
                    PlayerCommand::Pause => {
                        if let Some(ref s) = sink {
                            s.pause();
                            if let Some(start_time) = playback_start_time {
                                paused_position += start_time.elapsed().as_secs();
                            }
                            playback_start_time = None;

                            let mut status_guard = status.lock().unwrap();
                            status_guard.state = PlayerState::Paused;
                        }
                    }
                    PlayerCommand::Resume => {
                        if let Some(ref s) = sink {
                            s.play();
                            playback_start_time = Some(Instant::now());

                            let mut status_guard = status.lock().unwrap();
                            status_guard.state = PlayerState::Playing;
                        }
                    }
                    PlayerCommand::Stop => {
                        if let Some(s) = sink.take() {
                            s.stop();
                        }
                        playback_start_time = None;
                        paused_position = 0;

                        let mut status_guard = status.lock().unwrap();
                        status_guard.state = PlayerState::Stopped;
                        status_guard.current_position = 0;
                        status_guard.current_song = None;
                    }
                    PlayerCommand::SetVolume(volume) => {
                        if let Some(ref s) = sink {
                            s.set_volume(volume);
                        }
                        let mut status_guard = status.lock().unwrap();
                        status_guard.volume = volume;
                    }
                    PlayerCommand::SetShuffle(_shuffle_enabled) => {
                        // Store shuffle state for future playlist handling
                        // For now, just acknowledge the command
                        log::debug!("Shuffle mode updated");
                    }
                    PlayerCommand::SetRepeat(_repeat_mode) => {
                        // Store repeat mode for future playlist handling
                        // For now, just acknowledge the command
                        log::debug!("Repeat mode updated");
                    }
                    PlayerCommand::Quit => {
                        break;
                    }
                    PlayerCommand::Seek(_) => {
                        // Seeking is complex with rodio, skip for now
                        // Could be implemented with custom source
                    }
                }
            }

            // Update playback position
            if let Some(ref s) = sink {
                if s.empty() {
                    // Song finished
                    sink = None;
                    playback_start_time = None;
                    paused_position = 0;

                    let mut status_guard = status.lock().unwrap();
                    status_guard.state = PlayerState::Stopped;
                    status_guard.current_position = 0;
                    status_guard.current_song = None;
                } else if let Some(start_time) = playback_start_time {
                    let current_pos = paused_position + start_time.elapsed().as_secs();
                    let mut status_guard = status.lock().unwrap();
                    status_guard.current_position = current_pos.min(status_guard.total_duration);
                }
            }

            // Sleep to avoid busy waiting
            thread::sleep(Duration::from_millis(100));
        }

        // This code is unreachable but required for compilation
        #[allow(unreachable_code)]
        Ok(())
    }

    fn load_audio_file(
        path: &str,
        stream_handle: &OutputStream,
    ) -> Result<(Sink, u64)> {
        let file = File::open(path)
            .map_err(|e| LofiTurtleError::FileSystem(e))?;
        
        let buf_reader = BufReader::new(file);
        let decoder = Decoder::new(buf_reader)
            .map_err(|e| LofiTurtleError::UnsupportedFormat(format!("Failed to decode audio file '{}': {}", path, e)))?;

        // Get duration before consuming the decoder
        let total_duration = decoder.total_duration()
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let sink = Sink::connect_new(stream_handle.mixer());

        sink.append(decoder);
        sink.set_volume(0.7); // Default volume

        Ok((sink, total_duration))
    }
}
