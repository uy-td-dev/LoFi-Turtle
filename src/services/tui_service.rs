use crate::config::{Config, LayoutConfig};
use crate::error::{LofiTurtleError, Result};
use crate::library::{Database, MusicScanner};
use crate::ui::{draw_ui, App};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};

/// Service responsible for managing the terminal user interface
pub struct TuiService {
    config: Config,
    layout_config: LayoutConfig,
    terminal: Option<Terminal<CrosstermBackend<std::io::Stdout>>>,
}

impl TuiService {
    /// Create a new TUI service with the given configuration
    pub fn new(config: &Config, layout_config: &LayoutConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            layout_config: layout_config.clone(),
            terminal: None,
        })
    }

    /// Initialize the terminal interface
    fn initialize_terminal(&mut self) -> Result<()> {
        enable_raw_mode().map_err(|e| {
            LofiTurtleError::Terminal(format!("Failed to enable raw mode: {}", e))
        })?;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(|e| {
            LofiTurtleError::Terminal(format!("Failed to setup terminal: {}", e))
        })?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).map_err(|e| {
            LofiTurtleError::Terminal(format!("Failed to create terminal: {}", e))
        })?;

        self.terminal = Some(terminal);
        Ok(())
    }

    /// Restore the terminal to its original state
    fn restore_terminal(&mut self) -> Result<()> {
        if let Some(ref mut terminal) = self.terminal {
            disable_raw_mode().map_err(|e| {
                LofiTurtleError::Terminal(format!("Failed to disable raw mode: {}", e))
            })?;

            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            ).map_err(|e| {
                LofiTurtleError::Terminal(format!("Failed to restore terminal: {}", e))
            })?;

            terminal.show_cursor().map_err(|e| {
                LofiTurtleError::Terminal(format!("Failed to show cursor: {}", e))
            })?;
        }
        Ok(())
    }

    /// Initialize the music library if scanning is enabled
    fn initialize_library(&self) -> Result<()> {
        if self.config.no_scan {
            log::info!("Skipping library scan as requested");
            return Ok(());
        }

        log::info!("Initializing music library...");
        let mut database = Database::new(&self.config.database_path)?;
        
        println!("Scanning music directory: {}", self.config.music_dir.display());
        let scanner = MusicScanner::new();
        let songs = scanner.scan_directory(&self.config.music_dir)?;
        
        println!("Found {} songs. Adding to database...", songs.len());

        // Use bulk insert for better performance
        match database.insert_songs_bulk(&songs) {
            Ok(count) => println!("Successfully added {} songs to database", count),
            Err(e) => log::warn!("Failed to bulk insert songs: {}", e),
        }
        
        println!("Music library initialized successfully!");
        Ok(())
    }

    /// Run the main TUI application loop
    pub fn run(&mut self) -> Result<()> {
        // Initialize library first
        self.initialize_library()?;
        
        // Setup terminal
        self.initialize_terminal()?;
        
        // Ensure terminal is restored even if an error occurs
        let result = self.run_app_loop();
        
        // Always try to restore terminal
        if let Err(restore_err) = self.restore_terminal() {
            log::error!("Failed to restore terminal: {}", restore_err);
        }
        
        result
    }

    /// Main application event loop
    fn run_app_loop(&mut self) -> Result<()> {
        let terminal = self.terminal.as_mut().ok_or_else(|| {
            LofiTurtleError::Terminal("Terminal not initialized".to_string())
        })?;

        // Create app instance with layout config
        let mut app = App::new(&self.config, &self.layout_config)?;
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(self.config.tick_rate_ms);

        loop {
            // Draw UI
            terminal.draw(|f| draw_ui(f, &mut app)).map_err(|e| {
                LofiTurtleError::Terminal(format!("Failed to draw UI: {}", e))
            })?;

            // Handle events
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).map_err(|e| {
                LofiTurtleError::Terminal(format!("Failed to poll events: {}", e))
            })? {
                if let Event::Key(key) = event::read().map_err(|e| {
                    LofiTurtleError::Terminal(format!("Failed to read event: {}", e))
                })? {
                    if key.kind == KeyEventKind::Press {
                        if Self::handle_key_event(&mut app, key.code)? {
                            break; // User requested quit
                        }
                    }
                }
            }

            // Update app state on tick
            if last_tick.elapsed() >= tick_rate {
                app.update_playback_status();
                // Check for song completion and handle auto-advancement
                app.check_and_handle_song_completion()?;
                last_tick = Instant::now();
            }

            if app.should_quit() {
                break;
            }
        }

        Ok(())
    }

    /// Handle keyboard input events
    fn handle_key_event(app: &mut App, key_code: KeyCode) -> Result<bool> {
        use crate::ui::{InputMode, ActivePanel};
        
        match app.get_input_mode() {
            InputMode::Normal => {
                // Check for configured keybindings first
                let key_str = match key_code {
                    KeyCode::Char(' ') => "space".to_string(),
                    KeyCode::Char(c) => c.to_string(),
                    KeyCode::Enter => "enter".to_string(),
                    KeyCode::Tab => "tab".to_string(),
                    KeyCode::BackTab => "backtab".to_string(),
                    KeyCode::Esc => "esc".to_string(),
                    KeyCode::Backspace => "backspace".to_string(),
                    KeyCode::Up => "up".to_string(),
                    KeyCode::Down => "down".to_string(),
                    KeyCode::Left => "left".to_string(),
                    KeyCode::Right => "right".to_string(),
                    KeyCode::F(n) => format!("f{}", n),
                    KeyCode::Delete => "delete".to_string(),
                    KeyCode::Insert => "insert".to_string(),
                    KeyCode::Home => "home".to_string(),
                    KeyCode::End => "end".to_string(),
                    KeyCode::PageUp => "pageup".to_string(),
                    KeyCode::PageDown => "pagedown".to_string(),
                    _ => "".to_string(),
                };

                if !key_str.is_empty() {
                    if let Some(action) = app.layout_config.keybindings.get(&key_str) {
                        match action.as_str() {
                            "quit" => {
                                app.quit()?;
                                return Ok(true);
                            },
                            "toggle_play" => app.toggle_playback()?,
                            "next_track" => {
                                // Logic for next track
                                app.advance_to_next_song()?;
                            },
                            "previous_track" => {
                                // Logic for previous track
                                // For now just stop or restart current
                                app.stop_playback()?;
                            },
                            "move_up" => app.move_selection_up(),
                            "move_down" => app.move_selection_down(),
                            "select" => {
                                match app.state.active_panel {
                                    ActivePanel::Songs => {
                                        app.play_selected_song()?;
                                    }
                                    ActivePanel::Playlists => {
                                        app.play_selected_playlist()?;
                                    }
                                    _ => {}
                                }
                            },
                            "volume_up" => app.increase_volume()?,
                            "volume_down" => app.decrease_volume()?,
                            "switch_layout" => {
                                // Cycle layout logic could go here
                            },
                            "switch_theme" => app.cycle_theme(),
                            "reload_layout" => {
                                // Reload layout logic
                            },
                            "search" => app.enter_search_mode(),
                            "toggle_art" => app.toggle_album_art(),
                            _ => {}
                        }
                        return Ok(false);
                    }
                }

                // Fallback to hardcoded bindings if not found in config
                match key_code {
                    // Global controls
                    KeyCode::Char('q') => {
                        app.quit()?;
                        return Ok(true);
                    }
                    KeyCode::Tab => app.switch_to_next_panel(),
                    KeyCode::BackTab => app.switch_to_previous_panel(),
                    
                    // Navigation (Arrow keys)
                    KeyCode::Up => app.move_selection_up(),
                    KeyCode::Down => app.move_selection_down(),
                    KeyCode::Left => app.switch_to_previous_panel(),
                    KeyCode::Right => app.switch_to_next_panel(),
                    
                    // Vim-style navigation (hjkl)
                    KeyCode::Char('h') => app.switch_to_previous_panel(),
                    KeyCode::Char('j') => app.move_selection_down(),
                    KeyCode::Char('k') => app.move_selection_up(),
                    KeyCode::Char('l') => app.switch_to_library()?,
                    
                    // Navigation back
                    KeyCode::Backspace => app.switch_to_library()?,
                    KeyCode::Enter => {
                        match app.state.active_panel {
                            ActivePanel::Songs => {
                                app.play_selected_song()?;
                            }
                            ActivePanel::Playlists => {
                                // Play selected playlist (switch to it and start playing first song)
                                app.play_selected_playlist()?;
                            }
                            _ => {}
                        }
                    }
                    
                    // Playback controls
                    KeyCode::Char(' ') => app.toggle_playback()?,
                    KeyCode::Char('S') => app.toggle_shuffle()?,
                    KeyCode::Char('R') => app.cycle_repeat_mode()?,
                    KeyCode::Char('s') => app.stop_playback()?,
                    
                    // Volume controls
                    KeyCode::Char(']') => app.increase_volume()?,
                    KeyCode::Char('[') => app.decrease_volume()?,
                    
                    // Search and UI controls
                    KeyCode::Char('/') => app.enter_search_mode(),
                    KeyCode::Char('c') => app.clear_search(),
                    KeyCode::Char('a') => app.toggle_album_art(),
                    KeyCode::F(3) => app.cycle_theme(),

                    // Panel-specific controls
                    KeyCode::Char('n') => {
                        if matches!(app.state.active_panel, ActivePanel::Playlists) {
                            app.enter_playlist_create_mode();
                        }
                    }
                    KeyCode::Char('d') => {
                        if matches!(app.state.active_panel, ActivePanel::Playlists) {
                            app.delete_selected_playlist()?;
                        }
                    }
                    KeyCode::Char('e') => {
                        if matches!(app.state.active_panel, ActivePanel::Playlists) {
                            app.enter_playlist_edit_mode();
                        }
                    }
                    KeyCode::Char('+') => {
                        // Add selected song to selected playlist
                        if matches!(app.state.active_panel, ActivePanel::Songs) {
                            if let Some(song) = app.get_selected_song() {
                                let song_id = song.id.clone();
                                let _song_title = song.title.clone();
                                
                                // Get the currently selected playlist from the playlists panel
                                if !app.state.playlists.is_empty() {
                                    if let Some(playlist) = app.state.playlists.get(app.state.selected_playlist_index) {
                                        let playlist_name = playlist.name.clone();
                                        match app.add_song_to_playlist(&playlist_name, &song_id) {
                                            Ok(_) => {
                                                // Success - song added to playlist
                                                // The add_song_to_playlist method already handles reloading
                                            }
                                            Err(_e) => {
                                                // TODO: Add proper error display in UI
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Char('-') => {
                        // Remove selected song from current playlist or selected playlist
                        if matches!(app.state.active_panel, ActivePanel::Songs) {
                            if let Some(song) = app.get_selected_song() {
                                let song_id = song.id.clone();
                                
                                // First try to remove from current playlist if we're viewing one
                                if let Some(playlist_name) = app.get_current_playlist_name() {
                                    let playlist_name = playlist_name.to_string();
                                    let _ = app.remove_song_from_playlist(&playlist_name, &song_id);
                                    let _ = app.load_songs(); // Reload to reflect changes
                                } else if !app.state.playlists.is_empty() {
                                    // If not viewing a playlist, remove from the selected playlist
                                    if let Some(playlist) = app.state.playlists.get(app.state.selected_playlist_index) {
                                        let playlist_name = playlist.name.clone();
                                        let _ = app.remove_song_from_playlist(&playlist_name, &song_id);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            InputMode::Search => {
                match key_code {
                    KeyCode::Esc => app.exit_search_mode(),
                    KeyCode::Enter => {
                        app.play_selected_song()?;
                        app.exit_search_mode();
                    }
                    _ => {
                        app.handle_search_input(Event::Key(ratatui::crossterm::event::KeyEvent::new(
                            key_code,
                            ratatui::crossterm::event::KeyModifiers::empty(),
                        )))?;
                    }
                }
            }
            InputMode::PlaylistCreate | InputMode::PlaylistEdit => {
                match key_code {
                    KeyCode::Esc => app.exit_input_mode(),
                    KeyCode::Enter => {
                        app.confirm_playlist_action()?;
                    }
                    _ => {
                        app.handle_search_input(Event::Key(ratatui::crossterm::event::KeyEvent::new(
                            key_code,
                            ratatui::crossterm::event::KeyModifiers::empty(),
                        )))?;
                    }
                }
            }
        }
        
        Ok(false)
    }
}

impl Drop for TuiService {
    fn drop(&mut self) {
        // Ensure terminal is restored when service is dropped
        if let Err(e) = self.restore_terminal() {
            eprintln!("Failed to restore terminal in drop: {}", e);
        }
    }
}
