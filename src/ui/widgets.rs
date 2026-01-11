use crate::ui::{App, InputMode, ActivePanel, ViewMode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Gauge, List, ListItem, Paragraph, Clear,
    },
    Frame,
};
use std::collections::HashMap;

/// Helper to get color from hex string or name, defaulting to a fallback
fn get_color(color_str: Option<&String>, fallback: Color) -> Color {
    if let Some(c) = color_str {
        match c.to_lowercase().as_str() {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "gray" | "grey" => Color::Gray,
            "dark_gray" => Color::DarkGray,
            "white" => Color::White,
            s if s.starts_with('#') => {
                if let Ok(rgb) = u32::from_str_radix(&s[1..], 16) {
                    Color::Rgb(((rgb >> 16) & 0xFF) as u8, ((rgb >> 8) & 0xFF) as u8, (rgb & 0xFF) as u8)
                } else {
                    fallback
                }
            }
            _ => fallback,
        }
    } else {
        fallback
    }
}

pub fn draw_ui(f: &mut Frame, app: &mut App) {
    // Extract theme colors for easy access
    let theme = &app.layout_config.theme;
    let colors = theme.colors.as_ref();
    
    let primary_color = get_color(colors.and_then(|c| c.get("primary")), Color::Cyan);
    let secondary_color = get_color(colors.and_then(|c| c.get("secondary")), Color::Magenta);
    let _bg_color = get_color(colors.and_then(|c| c.get("background")), Color::Reset);
    let border_color = get_color(colors.and_then(|c| c.get("border")), Color::DarkGray);
    let highlight_color = get_color(colors.and_then(|c| c.get("highlight")), Color::Yellow);

    // Calculate layout using the layout engine
    let area = f.area();
    let layout_areas = match app.layout_engine.calculate_layout(area) {
        Ok(areas) => areas,
        Err(e) => {
            // Fallback if layout calculation fails
            log::error!("Layout calculation failed: {}", e);
            HashMap::new()
        }
    };

    // Draw widgets based on layout areas
    // We need to iterate over widgets without borrowing app immutably for the whole loop
    // because we need to pass mutable app to some draw functions (like draw_visual_panel)

    // First, collect the widgets we need to draw to avoid holding the borrow
    let widgets_to_draw: Vec<_> = app.layout_config.widgets.iter()
        .filter(|w| w.visible)
        .map(|w| (w.name.clone(), w.widget_type.clone()))
        .collect();

    for (name, widget_type) in widgets_to_draw {
        if let Some(area) = layout_areas.get(&name) {
            match widget_type {
                crate::ui::layout::WidgetType::Sidebar => {
                    draw_playlist_panel(f, app, *area, primary_color, secondary_color, border_color);
                },
                crate::ui::layout::WidgetType::PlaylistView => {
                    draw_song_list_panel(f, app, *area, primary_color, highlight_color, border_color);
                },
                crate::ui::layout::WidgetType::NowPlaying => {
                    draw_player_controls(f, app, *area, primary_color, secondary_color, border_color);
                },
                crate::ui::layout::WidgetType::AlbumArt => {
                    draw_visual_panel(f, app, *area, secondary_color, border_color);
                },
                crate::ui::layout::WidgetType::ProgressBar => {
                    // Progress bar is usually part of NowPlaying, but if separate:
                    draw_progress_bar(f, app, *area, primary_color);
                },
                crate::ui::layout::WidgetType::StatusBar => {
                    draw_status_bar(f, app, *area, border_color);
                },
                crate::ui::layout::WidgetType::SearchBox => {
                    draw_header(f, app, *area, primary_color, border_color);
                },
                _ => {}
            }
        }
    }

    // If no layout areas (fallback or empty config), use default hardcoded layout
    if layout_areas.is_empty() {
        draw_default_layout(f, app, primary_color, secondary_color, highlight_color, border_color);
    }

    // --- Modals ---
    if matches!(app.state.input_mode, InputMode::PlaylistCreate | InputMode::PlaylistEdit) {
        draw_input_modal(f, app, highlight_color);
    }

    // Draw scanning modal on top if scanning is in progress
    if app.state.is_scanning {
        draw_scanning_modal(f, app);
    }
}

fn draw_default_layout(f: &mut Frame, app: &mut App, primary: Color, secondary: Color, highlight: Color, border: Color) {
    // Main layout
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header / Search
            Constraint::Min(0),     // Main Content
            Constraint::Length(7),  // Player Controls (taller for better look)
        ])
        .split(f.area());

    // --- Header / Search Bar ---
    draw_header(f, app, main_chunks[0], primary, border);

    // --- Main Content Area ---
    // Check visible widgets to decide layout
    let visible_widgets: Vec<_> = app.layout_config.widgets.iter()
        .filter(|w| w.visible)
        .collect();
    
    let show_album_art = visible_widgets.iter().any(|w| w.name.contains("art"));

    let content_chunks = if show_album_art {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25), // Playlist/Library
                Constraint::Percentage(50), // Song List
                Constraint::Percentage(25), // Album Art & Visuals
            ])
            .split(main_chunks[1])
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // Playlist
                Constraint::Percentage(70), // Songs
            ])
            .split(main_chunks[1])
    };

    // Draw Panels
    draw_playlist_panel(f, app, content_chunks[0], primary, secondary, border);
    
    if content_chunks.len() > 1 {
        draw_song_list_panel(f, app, content_chunks[1], primary, highlight, border);
    }
    
    if content_chunks.len() > 2 {
        draw_visual_panel(f, app, content_chunks[2], secondary, border);
    }

    // --- Player Controls ---
    draw_player_controls(f, app, main_chunks[2], primary, secondary, border);
}

fn draw_scanning_modal(f: &mut Frame, app: &App) {
    let area = f.area();
    let popup_area = centered_rect(60, 25, area);

    f.render_widget(Clear, popup_area);

    let (processed, total) = app.state.scan_progress;
    let percentage = if total > 0 {
        (processed as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    let block = Block::default()
        .title("📀 Scanning Music Library...")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(popup_area);

    f.render_widget(block, popup_area);

    let text = Paragraph::new(format!("Processing file {} of {}...", processed, total))
        .alignment(Alignment::Center);
    f.render_widget(text, layout[0]);

    let progress_bar = Gauge::default()
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(percentage as u16)
        .label(format!("{:.0}%", percentage));
    f.render_widget(progress_bar, layout[2]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect, primary: Color, border: Color) {
    let title = match &app.state.view_mode {
        ViewMode::Library => " 🐢 Lofi Turtle Library ",
        ViewMode::Playlist(_name) => " 🐢 Playlist View ",
    };

    let border_style = if matches!(app.state.input_mode, InputMode::Search) {
        Style::default().fg(primary).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(border)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .title(Span::styled(title, Style::default().fg(primary).add_modifier(Modifier::BOLD)));

    if matches!(app.state.input_mode, InputMode::Search) {
        let mut textarea = app.state.search_textarea.clone();
        textarea.set_block(block);
        textarea.set_style(Style::default().fg(Color::White));
        textarea.set_cursor_style(Style::default().bg(primary));
        f.render_widget(&textarea, area);
    } else {
        // Just show the title or a hint when not searching
        let hint = if !app.state.search_query.is_empty() {
            format!("🔍 Filter: {}", app.state.search_query)
        } else {
            "Press '/' to search".to_string()
        };

        let p = Paragraph::new(hint)
            .block(block)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Left);
        f.render_widget(p, area);
    }
}

fn draw_playlist_panel(f: &mut Frame, app: &App, area: Rect, primary: Color, secondary: Color, border: Color) {
    let is_active = app.state.active_panel == ActivePanel::Playlists;
    let border_style = if is_active {
        Style::default().fg(primary).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(border)
    };

    let items: Vec<ListItem> = app.state.playlists.iter().enumerate().map(|(i, p)| {
        let is_selected = i == app.state.selected_playlist_index && is_active;
        let icon = if is_selected { "📂" } else { "📁" };

        let style = if is_selected {
            Style::default().fg(primary).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        ListItem::new(Line::from(vec![
            Span::styled(format!("{} ", icon), style),
            Span::styled(p.name.clone(), style),
            Span::styled(format!(" ({})", p.song_count()), Style::default().fg(Color::DarkGray)),
        ]))
    }).collect();

    // Add "Library" at top
    let mut all_items = vec![ListItem::new(Line::from(vec![
        Span::styled("📚 ", Style::default().fg(secondary)),
        Span::styled("All Music", if matches!(app.state.view_mode, ViewMode::Library) {
            Style::default().fg(secondary).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        }),
    ]))];
    all_items.extend(items);

    let list = List::new(all_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title(" Playlists "))
        .highlight_style(Style::default().bg(Color::DarkGray)); // Fallback highlight if needed

    f.render_widget(list, area);
}

fn draw_song_list_panel(f: &mut Frame, app: &App, area: Rect, primary: Color, highlight: Color, border: Color) {
    let is_active = app.state.active_panel == ActivePanel::Songs;
    let border_style = if is_active {
        Style::default().fg(primary).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(border)
    };

    let songs: Vec<ListItem> = app.state.filtered_songs.iter().enumerate().map(|(i, song)| {
        let is_selected = i == app.state.selected_song_index && is_active;
        let is_playing = app.get_current_song().map(|s| s.path == song.path).unwrap_or(false);

        let (icon, style) = if is_playing {
            ("▶ ", Style::default().fg(highlight).add_modifier(Modifier::BOLD))
        } else if is_selected {
            ("● ", Style::default().fg(primary).add_modifier(Modifier::BOLD))
        } else {
            ("  ", Style::default().fg(Color::Gray))
        };

        let title_width = (area.width as usize).saturating_sub(25); // Reserve space for duration/icon
        let title = format!("{:<width$}", song.display_name(), width = title_width);

        ListItem::new(Line::from(vec![
            Span::styled(icon, style),
            Span::styled(title, style),
            Span::styled(song.duration_formatted(), Style::default().fg(Color::DarkGray)),
        ]))
    }).collect();

    let title = match &app.state.view_mode {
        ViewMode::Library => format!(" Songs ({}) ", app.state.filtered_songs.len()),
        ViewMode::Playlist(n) => format!(" {} ({}) ", n, app.state.filtered_songs.len()),
    };

    let list = List::new(songs)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title(title));

    // We handle selection rendering manually above for better control,
    // but we need to pass a state if we want scrolling to work automatically.
    // Since AppState handles index, we just render the list centered around selection if possible
    // For now, simple rendering:

    // Calculate scroll offset to keep selection visible
    let height = area.height as usize - 2; // borders
    let _offset = if app.state.selected_song_index >= height {
        app.state.selected_song_index - height + 1
    } else {
        0
    };

    // Create a new list with offset (manual scrolling implementation for stateless widget)
    // In a real app we'd use ListState, but here we just slice for simplicity or rely on ratatui's state
    // Let's use the proper way:
    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.state.selected_song_index));
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_visual_panel(f: &mut Frame, app: &mut App, area: Rect, color: Color, border: Color) {
    let is_active = app.state.active_panel == ActivePanel::AlbumArt;
    let border_style = if is_active {
        Style::default().fg(color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(border)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .title(" Visuals ");

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // 1. Album Art (Top 70%)
    let art_area = Rect { height: (inner_area.height as f32 * 0.7) as u16, ..inner_area };

    if app.state.show_album_art {
        // Logic to render art... reusing existing logic but simplified
        if let Some(song) = app.get_current_song().cloned() {
             // Try to update/get art
             if let Ok(Some(art)) = app.update_album_art_with_dimensions(&song, art_area.width, art_area.height) {
                 let p = Paragraph::new(art).alignment(Alignment::Center);
                 f.render_widget(p, art_area);
             } else {
                 // Placeholder
                 let p = Paragraph::new("No Art").alignment(Alignment::Center).style(Style::default().fg(Color::DarkGray));
                 f.render_widget(p, art_area);
             }
        }
    } else {
        let p = Paragraph::new("Art Disabled\n(Press 'a')").alignment(Alignment::Center).style(Style::default().fg(Color::DarkGray));
        f.render_widget(p, centered_rect(80, 20, art_area));
    }

    // 2. Fake Visualizer (Bottom 30%)
    let viz_area = Rect { y: art_area.y + art_area.height, height: inner_area.height - art_area.height, ..inner_area };
    if app.state.playback_status.state == crate::audio::PlayerState::Playing {
        // Simple animated bars
        let bars = " ▂▃▅▆▇█▇▆▅▃▂ ";
        let repeated = bars.repeat(5);
        let p = Paragraph::new(repeated)
            .alignment(Alignment::Center)
            .style(Style::default().fg(color));
        f.render_widget(p, centered_rect(90, 50, viz_area));
    }
}

fn draw_player_controls(f: &mut Frame, app: &App, area: Rect, primary: Color, secondary: Color, border: Color) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .title(" Now Playing ");

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title/Artist
            Constraint::Length(1), // Progress Bar
            Constraint::Length(1), // Time & Status
            Constraint::Length(1), // Controls Help
        ])
        .margin(1)
        .split(inner);

    // 1. Song Info
    if let Some(song) = app.get_current_song() {
        let info = Line::from(vec![
            Span::styled("🎵 ", Style::default().fg(secondary)),
            Span::styled(&song.title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("  👤 ", Style::default().fg(secondary)),
            Span::styled(&song.artist, Style::default().fg(Color::Gray)),
        ]);
        f.render_widget(Paragraph::new(info).alignment(Alignment::Center), chunks[0]);
    } else {
        f.render_widget(Paragraph::new("Nothing Playing").alignment(Alignment::Center).style(Style::default().fg(Color::DarkGray)), chunks[0]);
    }

    // 2. Progress Bar
    let progress = if app.state.playback_status.total_duration > 0 {
        (app.state.playback_status.current_position as f64 / app.state.playback_status.total_duration as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(primary).bg(Color::DarkGray))
        .ratio(progress)
        .use_unicode(true); // Uses smooth blocks
    f.render_widget(gauge, chunks[1]);

    // 3. Time & Status Icons
    let time_str = format!("{} / {}",
        format_duration(app.state.playback_status.current_position),
        format_duration(app.state.playback_status.total_duration)
    );

    let status_icon = match app.state.playback_status.state {
        crate::audio::PlayerState::Playing => "▶",
        crate::audio::PlayerState::Paused => "⏸",
        crate::audio::PlayerState::Stopped => "⏹",
    };

    let shuffle_icon = if app.state.playback_state.shuffle { "🔀" } else { "➡" };
    let repeat_icon = match app.state.playback_state.repeat_mode {
        crate::models::RepeatMode::None => "➡",
        crate::models::RepeatMode::Single => "🔂",
        crate::models::RepeatMode::Playlist => "🔁",
    };
    let vol = (app.state.playback_status.volume * 100.0) as u8;
    let vol_icon = if vol == 0 { "🔇" } else if vol < 50 { "🔉" } else { "🔊" };

    let status_line = Line::from(vec![
        Span::styled(format!("{}  ", time_str), Style::default().fg(Color::Gray)),
        Span::styled(format!("{} ", status_icon), Style::default().fg(primary).add_modifier(Modifier::BOLD)),
        Span::raw("   "),
        Span::styled(format!("{} ", shuffle_icon), if app.state.playback_state.shuffle { Style::default().fg(secondary) } else { Style::default().fg(Color::DarkGray) }),
        Span::styled(format!("{} ", repeat_icon), if app.state.playback_state.repeat_mode != crate::models::RepeatMode::None { Style::default().fg(secondary) } else { Style::default().fg(Color::DarkGray) }),
        Span::raw("   "),
        Span::styled(format!("{} {}%", vol_icon, vol), Style::default().fg(Color::Gray)),
    ]);
    f.render_widget(Paragraph::new(status_line).alignment(Alignment::Center), chunks[2]);

    // 4. Quick Help
    let help = Span::styled(
        "Space:Play/Pause | Tab:Switch | /:Search | q:Quit",
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
    );
    f.render_widget(Paragraph::new(help).alignment(Alignment::Center), chunks[3]);
}

fn draw_progress_bar(f: &mut Frame, app: &App, area: Rect, primary: Color) {
    let progress = if app.state.playback_status.total_duration > 0 {
        (app.state.playback_status.current_position as f64 / app.state.playback_status.total_duration as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(primary).bg(Color::DarkGray))
        .ratio(progress)
        .use_unicode(true);
    f.render_widget(gauge, area);
}

fn draw_status_bar(f: &mut Frame, _app: &App, area: Rect, border: Color) {
    let help = Span::styled(
        "Space:Play/Pause | Tab:Switch | /:Search | q:Quit",
        Style::default().fg(border).add_modifier(Modifier::ITALIC)
    );
    f.render_widget(Paragraph::new(help).alignment(Alignment::Center), area);
}

fn draw_input_modal(f: &mut Frame, app: &App, highlight: Color) {
    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);
    
    let title = match app.state.input_mode {
        InputMode::PlaylistCreate => " Create Playlist ",
        InputMode::PlaylistEdit => " Edit Playlist ",
        _ => " Input ",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(highlight))
        .title(title);

    let mut textarea = app.state.playlist_name_textarea.clone();
    textarea.set_block(block);
    textarea.set_style(Style::default().fg(Color::White));
    f.render_widget(&textarea, area);
}

// Utils
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
