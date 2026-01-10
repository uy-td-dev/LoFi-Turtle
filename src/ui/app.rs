use crate::audio::{AudioPlayer, PlayerCommand, PlayerState, PlaybackStatus};
use crate::config::{Config, PersistentSettings, LayoutConfig};
use crate::error::{Result, LofiTurtleError};
use crate::library::Database;
use crate::models::{Song, Playlist, PlaybackState};
use crate::art::AlbumArtRenderer;
use crate::ui::theme::Themes;
use ratatui::crossterm::event::Event;
use std::time::Instant;
use tui_textarea::TextArea;

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Search,
    PlaylistCreate,
    PlaylistEdit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActivePanel {
    Playlists,
    Songs,
    AlbumArt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Library,
    Playlist(String),
}

#[derive(Debug)]
pub struct AppState {
    pub songs: Vec<Song>,
    pub filtered_songs: Vec<Song>,
    pub playlists: Vec<Playlist>,
    pub selected_song_index: usize,
    pub selected_playlist_index: usize,
    pub active_panel: ActivePanel,
    pub view_mode: ViewMode,
    pub search_query: String,
    pub input_mode: InputMode,
    pub search_textarea: TextArea<'static>,
    pub playlist_name_textarea: TextArea<'static>,
    pub playback_status: PlaybackStatus,
    pub playback_state: PlaybackState,
    pub show_album_art: bool,
    pub current_album_art: Option<String>,
    pub should_quit: bool,
    pub last_update: Instant,
    // New fields for scanning status
    pub is_scanning: bool,
    pub scan_progress: (usize, usize),
}

impl Default for AppState {
    fn default() -> Self {
        let mut search_textarea = TextArea::default();
        search_textarea.set_placeholder_text("Search songs...");
        
        let mut playlist_name_textarea = TextArea::default();
        playlist_name_textarea.set_placeholder_text("Enter playlist name...");
        
        Self {
            songs: Vec::new(),
            filtered_songs: Vec::new(),
            playlists: Vec::new(),
            selected_song_index: 0,
            selected_playlist_index: 0,
            active_panel: ActivePanel::Songs,
            view_mode: ViewMode::Library,
            search_query: String::new(),
            input_mode: InputMode::Normal,
            search_textarea,
            playlist_name_textarea,
            playback_status: PlaybackStatus::default(),
            playback_state: PlaybackState::default(),
            show_album_art: true,
            current_album_art: None,
            should_quit: false,
            last_update: Instant::now(),
            is_scanning: false,
            scan_progress: (0, 0),
        }
    }
}

pub struct App {
    pub state: AppState,
    pub database: Database,
    pub audio_player: AudioPlayer,
    pub album_art_renderer: AlbumArtRenderer,
    pub persistent_settings: PersistentSettings,
    pub layout_config: LayoutConfig,
}

impl App {
    pub fn new(config: &Config, layout_config: &LayoutConfig) -> Result<Self> {
        let database = Database::new(&config.database_path)?;
        let audio_player = AudioPlayer::new()?;
        let album_art_renderer = AlbumArtRenderer::new(config.album_art_config.clone());
        
        // Load persistent settings and set initial volume
        let persistent_settings = PersistentSettings::load();
        let initial_volume = persistent_settings.volume;
        
        let mut app = Self {
            state: AppState::default(),
            database,
            audio_player,
            album_art_renderer,
            persistent_settings,
            layout_config: layout_config.clone(),
        };
        
        // Set initial volume from persistent settings
        app.set_volume(initial_volume)?;

        // Load songs and playlists from database
        app.load_songs()?;
        app.load_playlists()?;
        
        // Apply config settings
        app.state.show_album_art = config.show_art;
        app.state.playback_state.shuffle = config.shuffle;
        app.state.playback_state.repeat_mode = config.repeat_mode.clone();
        
        Ok(app)
    }

    pub fn load_songs(&mut self) -> Result<()> {
        match &self.state.view_mode {
            ViewMode::Library => {
                self.state.songs = self.database.get_all_songs()?;
            }
            ViewMode::Playlist(playlist_name) => {
                // Get playlist by name to get its ID, then get songs
                if let Some(playlist) = self.database.get_playlist_by_name(playlist_name)? {
                    self.state.songs = self.database.get_playlist_songs(&playlist.id)?;
                } else {
                    self.state.songs = Vec::new();
                }
            }
        }
        self.update_filtered_songs();
        Ok(())
    }
    
    pub fn load_playlists(&mut self) -> Result<()> {
        self.state.playlists = self.database.get_all_playlists()?;
        
        // Ensure selected index is valid after loading
        if self.state.selected_playlist_index >= self.state.playlists.len() && !self.state.playlists.is_empty() {
            self.state.selected_playlist_index = self.state.playlists.len() - 1;
        } else if self.state.playlists.is_empty() {
            self.state.selected_playlist_index = 0;
        }
        
        Ok(())
    }

    pub fn update_filtered_songs(&mut self) {
        if self.state.search_query.is_empty() {
            // Optimization: Avoid cloning - just reference all songs
            self.state.filtered_songs = self.state.songs.clone();
        } else {
            // Optimization: Pre-lowercase query once to avoid repeated allocations
            let query_lower = self.state.search_query.to_lowercase();
            
            self.state.filtered_songs = self.state.songs
                .iter()
                .filter(|song| song.matches(&query_lower))
                .cloned()
                .collect();
        }

        // Reset selection if it's out of bounds
        if self.state.selected_song_index >= self.state.filtered_songs.len() {
            self.state.selected_song_index = 0;
        }
    }

    // Panel navigation methods
    pub fn switch_to_next_panel(&mut self) {
        self.state.active_panel = match self.state.active_panel {
            ActivePanel::Playlists => ActivePanel::Songs,
            ActivePanel::Songs => {
                if self.state.show_album_art {
                    ActivePanel::AlbumArt
                } else {
                    ActivePanel::Playlists
                }
            },
            ActivePanel::AlbumArt => ActivePanel::Playlists,
        };
    }
    
    pub fn switch_to_previous_panel(&mut self) {
        self.state.active_panel = match self.state.active_panel {
            ActivePanel::Playlists => {
                if self.state.show_album_art {
                    ActivePanel::AlbumArt
                } else {
                    ActivePanel::Songs
                }
            },
            ActivePanel::Songs => ActivePanel::Playlists,
            ActivePanel::AlbumArt => ActivePanel::Songs,
        };
    }

    // Selection movement methods
    pub fn move_selection_up(&mut self) {
        match self.state.active_panel {
            ActivePanel::Songs => {
                if !self.state.filtered_songs.is_empty() {
                    if self.state.selected_song_index > 0 {
                        self.state.selected_song_index -= 1;
                    } else {
                        self.state.selected_song_index = self.state.filtered_songs.len() - 1;
                    }
                }
            }
            ActivePanel::Playlists => {
                if !self.state.playlists.is_empty() {
                    if self.state.selected_playlist_index > 0 {
                        self.state.selected_playlist_index -= 1;
                    } else {
                        self.state.selected_playlist_index = self.state.playlists.len() - 1;
                    }
                }
            }
            ActivePanel::AlbumArt => {
                // Album art panel doesn't have selectable items
            }
        }
    }

    pub fn move_selection_down(&mut self) {
        match self.state.active_panel {
            ActivePanel::Songs => {
                if !self.state.filtered_songs.is_empty() {
                    if self.state.selected_song_index < self.state.filtered_songs.len() - 1 {
                        self.state.selected_song_index += 1;
                    } else {
                        self.state.selected_song_index = 0;
                    }
                }
            }
            ActivePanel::Playlists => {
                if !self.state.playlists.is_empty() {
                    if self.state.selected_playlist_index < self.state.playlists.len() - 1 {
                        self.state.selected_playlist_index += 1;
                    } else {
                        self.state.selected_playlist_index = 0;
                    }
                }
            }
            ActivePanel::AlbumArt => {
                // Album art panel doesn't have selectable items
            }
        }
    }

    pub fn play_selected_song(&mut self) -> Result<()> {
        match self.state.active_panel {
            ActivePanel::Songs => {
                if let Some(song) = self.state.filtered_songs.get(self.state.selected_song_index).cloned() {
                    self.audio_player.send_command(PlayerCommand::Play(song.path.clone()))?;
                    self.update_album_art(&song)?;
                }
            }
            ActivePanel::Playlists => {
                if let Some(playlist) = self.state.playlists.get(self.state.selected_playlist_index).cloned() {
                    self.switch_to_playlist(&playlist.name)?;
                }
            }
            ActivePanel::AlbumArt => {
                // Album art panel doesn't have playable items
            }
        }
        Ok(())
    }

    pub fn toggle_playback(&mut self) -> Result<()> {
        match self.state.playback_status.state {
            PlayerState::Playing => {
                self.audio_player.send_command(PlayerCommand::Pause)?;
            }
            PlayerState::Paused => {
                self.audio_player.send_command(PlayerCommand::Resume)?;
            }
            PlayerState::Stopped => {
                // If stopped, play the selected song
                self.play_selected_song()?;
            }
        }
        Ok(())
    }

    pub fn stop_playback(&mut self) -> Result<()> {
        self.audio_player.send_command(PlayerCommand::Stop)?;
        Ok(())
    }

    pub fn enter_search_mode(&mut self) {
        self.state.input_mode = InputMode::Search;
        self.state.search_textarea.move_cursor(tui_textarea::CursorMove::End);
    }

    pub fn exit_search_mode(&mut self) {
        self.state.input_mode = InputMode::Normal;
    }

    pub fn update_search_query(&mut self) {
        let new_query = self.state.search_textarea.lines().join("");
        if new_query != self.state.search_query {
            self.state.search_query = new_query;
            self.update_filtered_songs();
        }
    }

    pub fn clear_search(&mut self) {
        self.state.search_textarea = TextArea::default();
        self.state.search_textarea.set_placeholder_text("Search songs...");
        self.state.search_query.clear();
        self.update_filtered_songs();
    }

    pub fn update_playback_status(&mut self) {
        self.state.playback_status = self.audio_player.get_status();
        self.state.last_update = Instant::now();
        
        // Update album art if song changed
        if let Some(current_song) = self.get_current_song().cloned() {
            if self.state.show_album_art {
                let _ = self.update_album_art(&current_song);
            }
        }
    }

    pub fn quit(&mut self) -> Result<()> {
        self.state.should_quit = true;
        self.audio_player.send_command(PlayerCommand::Quit)?;
        Ok(())
    }

    pub fn get_current_song(&self) -> Option<&Song> {
        if let Some(ref current_path) = self.state.playback_status.current_song {
            self.state.songs.iter().find(|song| &song.path == current_path)
        } else {
            None
        }
    }

    #[allow(dead_code)] // Future feature: song selection info
    pub fn get_selected_song(&self) -> Option<&Song> {
        self.state.filtered_songs.get(self.state.selected_song_index)
    }

    /// Get the current input mode
    pub fn get_input_mode(&self) -> &InputMode {
        &self.state.input_mode
    }

    /// Check if the application should quit
    pub fn should_quit(&self) -> bool {
        self.state.should_quit
    }

    /// Handle search input events
    pub fn handle_search_input(&mut self, event: Event) -> Result<()> {
        match self.state.input_mode {
            InputMode::Search => {
                self.state.search_textarea.input(event);
                self.update_search_query();
            }
            InputMode::PlaylistCreate | InputMode::PlaylistEdit => {
                self.state.playlist_name_textarea.input(event);
            }
            _ => {}
        }
        Ok(())
    }
    
    // Playlist management methods
    pub fn switch_to_playlist(&mut self, playlist_name: &str) -> Result<()> {
        self.state.view_mode = ViewMode::Playlist(playlist_name.to_string());
        self.state.active_panel = ActivePanel::Songs;
        self.load_songs()?;
        Ok(())
    }
    
    pub fn play_selected_playlist(&mut self) -> Result<()> {
        if let Some(playlist) = self.state.playlists.get(self.state.selected_playlist_index) {
            let playlist_name = playlist.name.clone();
            
            // Switch to playlist view
            self.switch_to_playlist(&playlist_name)?;
            
            // If playlist has songs, play the first one
            if !self.state.songs.is_empty() {
                self.state.selected_song_index = 0;
                self.play_selected_song()?;
            } else {
            }
        }
        Ok(())
    }
    
    pub fn advance_to_next_song(&mut self) -> Result<()> {
        // Use enhanced PlaybackState for next song logic
        if !self.state.filtered_songs.is_empty() {
            let playlist_size = self.state.filtered_songs.len();
            
            if let Some(next_index) = self.state.playback_state.next_song_index(playlist_size) {
                self.state.selected_song_index = next_index;
                self.state.playback_state.set_current_song_index(next_index, playlist_size);
                self.play_selected_song()?;
            } else {
                // End of playlist with no repeat
                self.audio_player.send_command(PlayerCommand::Stop)?;
            }
        }
        Ok(())
    }
    
    pub fn check_and_handle_song_completion(&mut self) -> Result<()> {
        let status = self.audio_player.get_status();
        
        // Check if song just finished (state is Stopped and we were previously playing)
        if status.state == PlayerState::Stopped && status.current_song.is_none() {
            // Only auto-advance if we're in a playlist
            if matches!(self.state.view_mode, ViewMode::Playlist(_)) {
                self.advance_to_next_song()?;
            }
        }
        
        Ok(())
    }
    
    pub fn switch_to_library(&mut self) -> Result<()> {
        self.state.view_mode = ViewMode::Library;
        self.state.active_panel = ActivePanel::Songs;
        self.load_songs()?;
        Ok(())
    }
    
    pub fn create_playlist(&mut self, name: String, description: Option<String>) -> Result<()> {
        let playlist = Playlist::new(name, description);
        self.database.create_playlist(&playlist)?;
        self.load_playlists()?;
        Ok(())
    }
    
    pub fn delete_selected_playlist(&mut self) -> Result<()> {
        if let Some(playlist) = self.state.playlists.get(self.state.selected_playlist_index) {
            let _playlist_name = playlist.name.clone();
            let playlist_id = playlist.id.clone(); // Use the playlist ID, not the name
            
            match self.database.delete_playlist(&playlist_id) { // Pass ID instead of name
                Ok(deleted) => {
                    if deleted {
                        self.load_playlists()?;
                        
                        // Reset selection if out of bounds
                        if self.state.selected_playlist_index >= self.state.playlists.len() {
                            self.state.selected_playlist_index = if self.state.playlists.is_empty() { 0 } else { self.state.playlists.len() - 1 };
                        }
                    } else {
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        } else {
        }
        Ok(())
    }
    
    pub fn add_song_to_playlist(&mut self, playlist_name: &str, song_id: &str) -> Result<()> {
        // First get the playlist by name to get its ID
        if let Some(playlist) = self.database.get_playlist_by_name(playlist_name)? {
            // Debug: Print what we're trying to add
            
            self.database.add_song_to_playlist(&playlist.id, song_id, 0)?;
            
            // Debug: Verify the song was added
            if let Ok(_playlist_songs) = self.database.get_playlist_songs(&playlist.id) {
            }
            
            // Reload playlists to update song counts in the UI
            self.load_playlists()?;
            
            // If we're currently viewing this playlist, reload the songs
            if let ViewMode::Playlist(current_playlist) = &self.state.view_mode {
                if current_playlist == playlist_name {
                    self.load_songs()?;
                }
            }
        } else {
        }
        Ok(())
    }
    
    pub fn remove_song_from_playlist(&mut self, playlist_name: &str, song_id: &str) -> Result<()> {
        // First get the playlist by name to get its ID
        if let Some(playlist) = self.database.get_playlist_by_name(playlist_name)? {
            self.database.remove_song_from_playlist(&playlist.id, song_id)?;
            
            // Reload playlists to update song counts in the UI
            self.load_playlists()?;
            
            // If we're currently viewing this playlist, reload the songs
            if let ViewMode::Playlist(current_playlist) = &self.state.view_mode {
                if current_playlist == playlist_name {
                    self.load_songs()?;
                }
            }
        }
        Ok(())
    }
    
    // Enhanced playback mode controls with fair randomization
    pub fn toggle_shuffle(&mut self) -> Result<()> {
        let playlist_size = self.state.filtered_songs.len();
        self.state.playback_state.toggle_shuffle(playlist_size);
        
        // Save to persistent settings
        self.save_playback_settings()?;
        Ok(())
    }
    
    pub fn cycle_repeat_mode(&mut self) -> Result<()> {
        self.state.playback_state.cycle_repeat_mode();
        
        // Save to persistent settings
        self.save_playback_settings()?;
        Ok(())
    }

    /// Save playback settings (shuffle, repeat, volume) to persistent storage
    pub fn save_playback_settings(&self) -> Result<()> {
        let settings = PersistentSettings {
            volume: self.state.playback_status.volume,
            shuffle: self.state.playback_state.shuffle,
            repeat_mode: self.state.playback_state.repeat_mode,
        };
        settings.save()
    }

    
    // Album art methods
    pub fn toggle_album_art(&mut self) {
        self.state.show_album_art = !self.state.show_album_art;
        if !self.state.show_album_art {
            self.state.current_album_art = None;
            // If currently on album art panel, switch to songs panel
            if matches!(self.state.active_panel, ActivePanel::AlbumArt) {
                self.state.active_panel = ActivePanel::Songs;
            }
        } else {
            // When enabling album art, immediately update with current song
            if let Some(current_song) = self.get_current_song().cloned() {
                let _ = self.update_album_art(&current_song);
            } else if let Some(selected_song) = self.get_selected_song().cloned() {
                let _ = self.update_album_art(&selected_song);
            }
        }
    }
    
    pub fn update_album_art(&mut self, song: &Song) -> Result<()> {
        if self.state.show_album_art {
            match self.album_art_renderer.render_album_art_from_file(&song.path) {
                Ok(art) => self.state.current_album_art = Some(art),
                Err(_) => self.state.current_album_art = None,
            }
        }
        Ok(())
    }

    /// Update album art with specific dimensions for dynamic scaling
    pub fn update_album_art_with_dimensions(&mut self, song: &Song, panel_width: u16, panel_height: u16) -> Result<Option<String>> {
        if !self.state.show_album_art {
            return Ok(None);
        }

        // Extract album art from the song file
        match self.album_art_renderer.extract_album_art(&song.path)? {
            Some(image_data) => {
                // Render with dynamic dimensions
                let art = match self.album_art_renderer.render_album_art_for_panel(&image_data, panel_width, panel_height) {
                    Ok(art) => art,
                    Err(e) => return Err(LofiTurtleError::from(std::io::Error::new(std::io::ErrorKind::Other, format!("Album art rendering error: {}", e)))),
                };
                self.state.current_album_art = Some(art.clone());
                Ok(Some(art))
            }
            None => {
                // Generate placeholder with dynamic dimensions
                let placeholder = self.album_art_renderer.generate_placeholder_for_panel(panel_width, panel_height);
                self.state.current_album_art = Some(placeholder.clone());
                Ok(Some(placeholder))
            }
        }
    }

    /// Generate album art placeholder with dynamic dimensions
    #[allow(dead_code)]
    pub fn generate_album_art_placeholder(&mut self, panel_width: u16, panel_height: u16) -> String {
        if !self.state.show_album_art {
            return String::new();
        }
        
        self.album_art_renderer.generate_placeholder_for_panel(panel_width, panel_height)
    }
    
    // Input mode management
    pub fn enter_playlist_create_mode(&mut self) {
        self.state.input_mode = InputMode::PlaylistCreate;
        self.state.playlist_name_textarea = TextArea::default();
        self.state.playlist_name_textarea.set_placeholder_text("Enter playlist name...");
    }
    
    pub fn enter_playlist_edit_mode(&mut self) {
        if let Some(playlist) = self.state.playlists.get(self.state.selected_playlist_index) {
            self.state.input_mode = InputMode::PlaylistEdit;
            self.state.playlist_name_textarea = TextArea::default();
            self.state.playlist_name_textarea.insert_str(&playlist.name);
        }
    }
    
    pub fn exit_input_mode(&mut self) {
        self.state.input_mode = InputMode::Normal;
    }
    
    pub fn confirm_playlist_action(&mut self) -> Result<()> {
        let playlist_name = self.state.playlist_name_textarea.lines().join("");
        if !playlist_name.trim().is_empty() {
            match self.state.input_mode {
                InputMode::PlaylistCreate => {
                    self.create_playlist(playlist_name.trim().to_string(), None)?;
                }
                InputMode::PlaylistEdit => {
                    // For now, we'll implement rename functionality later
                    // This would require database schema changes
                }
                _ => {}
            }
        }
        self.exit_input_mode();
        Ok(())
    }
    
    // Utility methods
    pub fn get_current_playlist_name(&self) -> Option<&str> {
        match &self.state.view_mode {
            ViewMode::Playlist(name) => Some(name),
            ViewMode::Library => None,
        }
    }
    
    
    // Volume control methods
    pub fn increase_volume(&mut self) -> Result<()> {
        let current_volume = self.state.playback_status.volume;
        let new_volume = (current_volume + 0.1).min(1.0);
        self.set_volume(new_volume)
    }
    
    pub fn decrease_volume(&mut self) -> Result<()> {
        let current_volume = self.state.playback_status.volume;
        let new_volume = (current_volume - 0.1).max(0.0);
        self.set_volume(new_volume)
    }
    
    pub fn set_volume(&mut self, volume: f32) -> Result<()> {
        let clamped_volume = volume.clamp(0.0, 1.0);
        self.audio_player.send_command(PlayerCommand::SetVolume(clamped_volume))?;
        self.state.playback_status.volume = clamped_volume;
        
        // Save volume to persistent settings
        self.persistent_settings.update_volume(clamped_volume)?;
        
        Ok(())
    }

    /// Cycle through available themes
    pub fn cycle_theme(&mut self) {
        let themes = Themes::all();
        let current_theme_name = &self.layout_config.theme.name;

        // Find current theme index
        let current_index = themes.iter()
            .position(|t| &t.name == current_theme_name)
            .unwrap_or(0);

        // Calculate next index
        let next_index = (current_index + 1) % themes.len();

        // Update theme
        self.layout_config.theme = themes[next_index].clone();
    }
    
}
