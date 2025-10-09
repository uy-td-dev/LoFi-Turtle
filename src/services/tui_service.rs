use crate::config::Config;
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
use tokio::task;

/// Service responsible for managing the terminal user interface
pub struct TuiService {
    config: Config,
    terminal: Option<Terminal<CrosstermBackend<std::io::Stdout>>>,
}

impl TuiService {
    /// Create a new TUI service with the given configuration
    pub fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
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

    /// Scans the music library and populates the database. This is a blocking operation.
    fn scan_and_populate_library(config: &Config) -> Result<()> {
        if config.no_scan {
            log::info!("Skipping library scan as requested");
            return Ok(());
        }

        log::info!("Initializing music library...");
        let database = Database::new(&config.database_path)?;

        log::info!("Scanning music directory: {}", config.music_dir.display());
        let scanner = MusicScanner::new();
        let songs = scanner.scan_directory(&config.music_dir)?;

        log::info!("Found {} songs. Adding to database...", songs.len());
        let mut error_count = 0;

        for song in &songs {
            if let Err(e) = database.insert_song(song) {
                log::warn!("Failed to insert song {}: {}", song.path, e);
                error_count += 1;
            }
        }

        if error_count > 0 {
            log::warn!("Warning: {} songs failed to be added to database", error_count);
        }

        log::info!("Music library initialized successfully!");
        Ok(())
    }

    /// Run the main TUI application loop
    pub async fn run(&mut self) -> Result<()> {
        // Clone config for the background task
        let config = self.config.clone();
        if !self.config.no_scan {
            // Spawn a background task for library initialization
            tokio::spawn(async move {
                let result = task::spawn_blocking(move || {
                    Self::scan_and_populate_library(&config)
                })
                .await;

                match result {
                    Ok(Ok(_)) => log::info!("Background library scan finished successfully."),
                    Ok(Err(e)) => log::error!("Library scanning failed: {}", e),
                    Err(e) => log::error!("Library scanning task panicked: {}", e),
                }
            });
        }
        
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

        // Create app instance
        let mut app = App::new(&self.config)?;
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
