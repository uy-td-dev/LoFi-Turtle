use crate::ui::{App, InputMode, ActivePanel, ViewMode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Gauge, List, ListItem, Paragraph, Wrap, Clear,
    },
    Frame,
};

pub fn draw_ui(f: &mut Frame, app: &mut App) {
    // Main layout: search bar, content area, control panel
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Search bar
            Constraint::Min(0),     // Content area
            Constraint::Length(6),  // Enhanced control panel
        ])
        .split(f.area());

    // Content area layout: playlists, songs, and optionally album art
    let (content_chunks, show_album_art) = if app.state.show_album_art {
        // Three panels: playlist, songs, album art
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25), // Playlist panel
                Constraint::Percentage(50), // Song list panel
                Constraint::Percentage(25), // Album art panel
            ])
            .split(main_chunks[1]);
        (chunks, true)
    } else {
        // Two panels: playlist and songs (no album art)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33), // Playlist panel (wider)
                Constraint::Percentage(67), // Song list panel (wider)
            ])
            .split(main_chunks[1]);
        (chunks, false)
    };

    // Draw all panels
    draw_search_bar(f, app, main_chunks[0]);
    draw_playlist_panel(f, app, content_chunks[0]);
    draw_song_list_panel(f, app, content_chunks[1]);
    
    // Only draw album art panel if enabled
    if show_album_art && content_chunks.len() > 2 {
        draw_album_art_panel(f, app, content_chunks[2]);
    }
    
    draw_enhanced_control_panel(f, app, main_chunks[2]);
    
    // Draw modal dialogs if in input modes
    match app.state.input_mode {
        InputMode::PlaylistCreate | InputMode::PlaylistEdit => {
            draw_playlist_input_modal(f, app);
        }
        _ => {}
    }
}

fn draw_search_bar(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let title = match &app.state.view_mode {
        ViewMode::Library => "Search Library".to_string(),
        ViewMode::Playlist(name) => format!("Search Playlist: {}", name),
    };
    
    let search_block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(match app.state.input_mode {
            InputMode::Search => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        });

    let mut textarea = app.state.search_textarea.clone();
    textarea.set_block(search_block);
    f.render_widget(&textarea, area);
}

fn draw_playlist_panel(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let playlists: Vec<ListItem> = app
        .state
        .playlists
        .iter()
        .enumerate()
        .map(|(i, playlist)| {
            let song_count = format!(" ({} songs)", playlist.song_count());
            let content = if i == app.state.selected_playlist_index && app.state.active_panel == ActivePanel::Playlists {
                Line::from(vec![
                    Span::styled(">> ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled(&playlist.name, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled(song_count, Style::default().fg(Color::Gray)),
                ])
            } else {
                Line::from(vec![
                    Span::raw("   "),
                    Span::raw(&playlist.name),
                    Span::styled(song_count, Style::default().fg(Color::Gray)),
                ])
            };
            ListItem::new(content)
        })
        .collect();

    // Add "Library" option at the top
    let mut all_items = vec![ListItem::new(
        if matches!(app.state.view_mode, ViewMode::Library) && app.state.active_panel == ActivePanel::Playlists {
            Line::from(vec![
                Span::styled(">> ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled("📚 All Songs", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ])
        } else {
            Line::from(vec![
                Span::raw("   "),
                Span::styled("📚 All Songs", Style::default().fg(Color::Cyan)),
            ])
        }
    )];
    all_items.extend(playlists);

    let playlist_list = List::new(all_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Playlists")
                .border_style(match app.state.active_panel {
                    ActivePanel::Playlists => Style::default().fg(Color::Green),
                    _ => Style::default(),
                })
        );

    f.render_widget(playlist_list, area);
}

fn draw_song_list_panel(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let songs: Vec<ListItem> = app
        .state
        .filtered_songs
        .iter()
        .enumerate()
        .map(|(i, song)| {
            let display_name = song.display_name();
            // Performance optimization: Use cached duration string
            let duration_text = format!(" [{}]", song.duration_formatted());
            let content = if i == app.state.selected_song_index && app.state.active_panel == ActivePanel::Songs {
                Line::from(vec![
                    Span::styled(">> ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(display_name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(duration_text, Style::default().fg(Color::Gray)),
                ])
            } else {
                Line::from(vec![
                    Span::raw("   "),
                    Span::raw(display_name),
                    Span::styled(duration_text, Style::default().fg(Color::Gray)),
                ])
            };
            ListItem::new(content)
        })
        .collect();

    let title = match &app.state.view_mode {
        ViewMode::Library => format!("Songs ({}/{})", 
            app.state.filtered_songs.len(), 
            app.state.songs.len()
        ),
        ViewMode::Playlist(name) => format!("{} ({}/{})", 
            name,
            app.state.filtered_songs.len(), 
            app.state.songs.len()
        ),
    };
    
    let songs_list = List::new(songs)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(match app.state.active_panel {
                    ActivePanel::Songs => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                })
        );

    f.render_widget(songs_list, area);
}

fn draw_album_art_panel(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Album Art")
        .border_style(match app.state.active_panel {
            ActivePanel::AlbumArt => Style::default().fg(Color::Magenta),
            _ => Style::default(),
        });

    if app.state.show_album_art {
        // Create a constrained area for 80% width within the panel (CSS formula)
        let inner_area = area.inner(Margin { vertical: 1, horizontal: 1 });
        let art_width = ((inner_area.width as f32) * 0.8) as u16;
        let art_height = inner_area.height;
        
        // Center the constrained area within the panel
        let x_offset = (inner_area.width.saturating_sub(art_width)) / 2;
        let constrained_area = ratatui::layout::Rect {
            x: inner_area.x + x_offset,
            y: inner_area.y,
            width: art_width,
            height: art_height,
        };

        // Check if we need to regenerate album art with new dimensions
        let current_song = app.get_current_song().cloned();
        if let Some(song) = current_song {
            // Update album art with constrained dimensions (80% width)
            if let Ok(updated_art) = app.update_album_art_with_dimensions(&song, constrained_area.width, constrained_area.height) {
                if let Some(ref art) = updated_art {
                    let art_paragraph = Paragraph::new(art.clone())
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: true });
                    f.render_widget(art_paragraph, constrained_area);
                    
                    // Draw the main block border
                    f.render_widget(block, area);
                    return;
                }
            }
        }
        
        // Fallback to existing art or placeholder
        if let Some(ref art) = app.state.current_album_art {
            let art_paragraph = Paragraph::new(art.clone())
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            f.render_widget(art_paragraph, constrained_area);
            f.render_widget(block, area);
        } else {
            // Generate placeholder with constrained dimensions (80% width)
            let placeholder_art = app.generate_album_art_placeholder(constrained_area.width, constrained_area.height);
            let placeholder = Paragraph::new(placeholder_art)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
            f.render_widget(placeholder, constrained_area);
            f.render_widget(block, area);
        }
    } else {
        let disabled_msg = Paragraph::new("Album art\ndisabled\n\nPress 'a' to\nenable")
            .block(block)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(disabled_msg, area);
    }
}

fn draw_enhanced_control_panel(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let control_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Current song info
            Constraint::Length(1),
            Constraint::Length(1),// Progress bar
            Constraint::Length(1),  // Controls info
            Constraint::Length(1),  // Status,
            Constraint::Length(1)// Playback modes (shuffle/repeat)
        ])
        .split(area.inner(Margin { vertical: 1, horizontal: 1 }));

    // Current song info
    let current_song_text = if let Some(song) = app.get_current_song() {
        format!("♪ {} - {}", song.title, song.artist)
    } else {
        "No song playing".to_string()
    };

    let current_song = Paragraph::new(current_song_text)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    f.render_widget(current_song, control_chunks[0]);

    // Progress bar
    let progress = if app.state.playback_status.total_duration > 0 {
        (app.state.playback_status.current_position as f64 / app.state.playback_status.total_duration as f64) * 100.0
    } else {
        0.0
    };

    let progress_label = format!(
        "{} / {}",
        format_duration(app.state.playback_status.current_position),
        format_duration(app.state.playback_status.total_duration)
    );

    let progress_bar = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(Color::Green))
        .percent(progress as u16)
        .label(progress_label);
    f.render_widget(progress_bar, control_chunks[2]);

    // Playback modes (shuffle/repeat) and volume - using enhanced PlaybackState
    let shuffle_icon = if app.state.playback_state.shuffle { "🔀 ON" } else { "🔀 OFF" };
    let repeat_icon = match app.state.playback_state.repeat_mode {
        crate::models::RepeatMode::None => "🔁 OFF",
        crate::models::RepeatMode::Single => "🔂 SINGLE",
        crate::models::RepeatMode::Playlist => "🔁 PLAYLIST",
    };
    
    let volume_percent = (app.state.playback_status.volume * 100.0) as u8;
    let volume_icon = if volume_percent == 0 {
        "🔇"
    } else if volume_percent < 33 {
        "🔈"
    } else if volume_percent < 67 {
        "🔉"
    } else {
        "🔊"
    };
    
    let modes_text = format!("Shuffle: {}  |  Repeat: {}  |  Volume: {} {}%", 
                            shuffle_icon, repeat_icon, volume_icon, volume_percent);
    let modes = Paragraph::new(modes_text)
        .style(Style::default().fg(Color::Magenta))
        .alignment(Alignment::Center);
    f.render_widget(modes, control_chunks[5]);

    // Controls info
    let controls_text = match app.state.input_mode {
        InputMode::Normal => {
            match app.state.active_panel {
                ActivePanel::Playlists => "hjkl/↑↓←→: Navigate | Tab: Switch panels | Enter: Select | Backspace: Back | [/]: Volume | +/-: Add/Remove songs | n: New | d: Delete | a: Toggle art | q: Quit",
                ActivePanel::Songs => "hjkl/↑↓←→: Navigate | Tab: Switch panels | Enter: Play | Space: Play/Pause | S: Shuffle | R: Repeat | [/]: Volume | +/-: Add/Remove to playlist | Backspace: Back | /: Search | a: Toggle art | q: Quit",
                ActivePanel::AlbumArt => "hjkl/↑↓←→: Navigate | Tab: Switch panels | [/]: Volume | Backspace: Back | a: Toggle album art | q: Quit",
            }
        },
        InputMode::Search => "Type to search | Esc: Exit search | Enter: Play selected",
        InputMode::PlaylistCreate => "Enter playlist name | Enter: Create | Esc: Cancel",
        InputMode::PlaylistEdit => "Edit playlist name | Enter: Save | Esc: Cancel",
    };

    let controls = Paragraph::new(controls_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(controls, control_chunks[3]);

    // Status
    let status_text = match app.state.playback_status.state {
        crate::audio::PlayerState::Playing => "▶ Playing",
        crate::audio::PlayerState::Paused => "⏸ Paused",
        crate::audio::PlayerState::Stopped => "⏹ Stopped",
    };

    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(status, control_chunks[4]);

    // Draw border around control panel
    let control_block = Block::default()
        .borders(Borders::ALL)
        .title("Controls");
    f.render_widget(control_block, area);
}

fn draw_playlist_input_modal(f: &mut Frame, app: &App) {
    let area = f.area();
    let popup_area = centered_rect(50, 20, area);
    
    // Clear the area
    f.render_widget(Clear, popup_area);
    
    let title = match app.state.input_mode {
        InputMode::PlaylistCreate => "Create New Playlist",
        InputMode::PlaylistEdit => "Edit Playlist",
        _ => "Playlist",
    };
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Yellow));
    
    let mut textarea = app.state.playlist_name_textarea.clone();
    textarea.set_block(block);
    f.render_widget(&textarea, popup_area);
}

// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn format_duration(seconds: u64) -> String {
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}
