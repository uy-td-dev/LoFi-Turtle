// Minimal demo with hardcoded layouts - no file system access at all
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::time::{Duration, Instant};

#[derive(Clone)]
enum LayoutType {
    Default,
    Compact,
    Minimal,
}

#[derive(Clone)]
enum ThemeType {
    Dark,
    Light,
    Synthwave,
    Forest,
}

struct MinimalDemo {
    current_layout: LayoutType,
    current_theme: ThemeType,
    playlist: Vec<String>,
    selected: usize,
    playing: bool,
    volume: f32,
    progress: f32,
    show_help: bool,
    status: Option<(String, Instant)>,
}

impl MinimalDemo {
    fn new() -> Self {
        Self {
            current_layout: LayoutType::Default,
            current_theme: ThemeType::Dark,
            playlist: vec![
                "Lofi Hip Hop - Chill Vibes.mp3".to_string(),
                "Study Session - Focus Beats.mp3".to_string(),
                "Rain Sounds - Peaceful Night.mp3".to_string(),
                "Coffee Shop Ambience.mp3".to_string(),
            ],
            selected: 0,
            playing: true,
            volume: 0.7,
            progress: 0.45,
            show_help: false,
            status: Some(("F1:Help | F2:Layout | F3:Theme | Space:Play | Q:Quit".to_string(), Instant::now())),
        }
    }

    fn run<B: ratatui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Box<dyn std::error::Error>> {
        let mut last_tick = Instant::now();

        loop {
            terminal.draw(|f| self.draw(f))?;

            if crossterm::event::poll(Duration::from_millis(250))? {
                if let Event::Key(key) = event::read()? {
                    if self.handle_key(key)? {
                        break;
                    }
                }
            }

            if last_tick.elapsed() >= Duration::from_millis(250) {
                if self.playing {
                    self.progress += 0.01;
                    if self.progress > 1.0 {
                        self.progress = 0.0;
                    }
                }
                last_tick = Instant::now();
            }
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            KeyCode::Char(' ') => {
                self.playing = !self.playing;
                self.set_status(if self.playing { "Playing" } else { "Paused" }.to_string());
            }
            KeyCode::F(1) => self.show_help = !self.show_help,
            KeyCode::F(2) => self.switch_layout(),
            KeyCode::F(3) => self.switch_theme(),
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected < self.playlist.len() - 1 {
                    self.selected += 1;
                }
            }
            KeyCode::Enter => {
                self.set_status(format!("Selected: {}", self.playlist[self.selected]));
            }
            KeyCode::Char('+') => {
                self.volume = (self.volume + 0.1).min(1.0);
                self.set_status(format!("Volume: {}%", (self.volume * 100.0) as u8));
            }
            KeyCode::Char('-') => {
                self.volume = (self.volume - 0.1).max(0.0);
                self.set_status(format!("Volume: {}%", (self.volume * 100.0) as u8));
            }
            _ => {}
        }
        Ok(false)
    }

    fn switch_layout(&mut self) {
        self.current_layout = match self.current_layout {
            LayoutType::Default => LayoutType::Compact,
            LayoutType::Compact => LayoutType::Minimal,
            LayoutType::Minimal => LayoutType::Default,
        };
        let name = match self.current_layout {
            LayoutType::Default => "Default Layout",
            LayoutType::Compact => "Compact Layout",
            LayoutType::Minimal => "Minimal Layout",
        };
        self.set_status(format!("Switched to: {}", name));
    }

    fn switch_theme(&mut self) {
        self.current_theme = match self.current_theme {
            ThemeType::Dark => ThemeType::Light,
            ThemeType::Light => ThemeType::Synthwave,
            ThemeType::Synthwave => ThemeType::Forest,
            ThemeType::Forest => ThemeType::Dark,
        };
        let name = match self.current_theme {
            ThemeType::Dark => "Dark Theme",
            ThemeType::Light => "Light Theme",
            ThemeType::Synthwave => "Synthwave Theme",
            ThemeType::Forest => "Forest Theme",
        };
        self.set_status(format!("Switched to: {}", name));
    }

    fn draw(&mut self, f: &mut Frame) {
        let area = f.area();

        match self.current_layout {
            LayoutType::Default => self.draw_default_layout(f, area),
            LayoutType::Compact => self.draw_compact_layout(f, area),
            LayoutType::Minimal => self.draw_minimal_layout(f, area),
        }

        if self.show_help {
            self.draw_help_overlay(f, area);
        }
    }

    fn draw_default_layout(&self, f: &mut Frame, area: Rect) {
        // Three-panel layout: sidebar | playlist | now_playing
        // Bottom: status bar
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);

        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25), // sidebar
                Constraint::Fill(1),        // playlist
                Constraint::Percentage(30), // now playing
            ])
            .split(main_layout[0]);

        self.draw_sidebar(f, content_layout[0]);
        self.draw_playlist(f, content_layout[1], true, Some("Current Playlist"));
        self.draw_now_playing(f, content_layout[2]);
        self.draw_status_bar(f, main_layout[1]);
    }

    fn draw_compact_layout(&self, f: &mut Frame, area: Rect) {
        // Simple layout: playlist + status
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);

        self.draw_playlist(f, layout[0], true, Some("Playlist"));
        self.draw_status_bar(f, layout[1]);
    }

    fn draw_minimal_layout(&self, f: &mut Frame, area: Rect) {
        // Ultra-minimal: just playlist, no borders
        self.draw_playlist(f, area, false, None);
    }

    fn draw_sidebar(&self, f: &mut Frame, area: Rect) {
        let items = vec![
            ListItem::new("🎵 Library"),
            ListItem::new("📂 Playlists"),
            ListItem::new("⭐ Favorites"),
            ListItem::new("🕒 History"),
        ];

        let list = List::new(items)
            .block(
                Block::default()
                    .title("Library")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(self.get_border_style()),
            )
            .style(self.get_normal_style());

        f.render_widget(list, area);
    }

    fn draw_playlist(&self, f: &mut Frame, area: Rect, with_border: bool, title: Option<&str>) {
        let items: Vec<ListItem> = self
            .playlist
            .iter()
            .enumerate()
            .map(|(i, track)| {
                let prefix = if i == self.selected { "▶ " } else { "  " };
                let style = if i == self.selected {
                    self.get_selected_style()
                } else {
                    self.get_normal_style()
                };
                ListItem::new(format!("{}{}", prefix, track)).style(style)
            })
            .collect();

        let mut state = ListState::default();
        state.select(Some(self.selected));

        let block = if with_border {
            Block::default()
                .title(title.unwrap_or("Playlist"))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(self.get_border_style())
        } else {
            Block::default()
        };

        let list = List::new(items)
            .block(block)
            .highlight_style(self.get_highlight_style());

        f.render_stateful_widget(list, area, &mut state);
    }

    fn draw_now_playing(&self, f: &mut Frame, area: Rect) {
        let current_track = &self.playlist[self.selected];
        let content = vec![
            Line::from(vec![
                Span::styled("♪ ", self.get_playing_style()),
                Span::styled(current_track, self.get_normal_style()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Status: ", self.get_secondary_style()),
                Span::styled(
                    if self.playing { "Playing" } else { "Paused" },
                    if self.playing {
                        self.get_playing_style()
                    } else {
                        self.get_paused_style()
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled("Volume: ", self.get_secondary_style()),
                Span::styled(
                    format!("{}%", (self.volume * 100.0) as u8),
                    self.get_normal_style(),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Progress: ", self.get_secondary_style()),
                Span::styled(
                    format!("{}%", (self.progress * 100.0) as u8),
                    self.get_normal_style(),
                ),
            ]),
        ];

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .title("Now Playing")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(self.get_border_style()),
            );

        f.render_widget(paragraph, area);
    }

    fn draw_status_bar(&self, f: &mut Frame, area: Rect) {
        let status_text = if let Some((ref message, timestamp)) = self.status {
            if timestamp.elapsed() < Duration::from_secs(3) {
                message.clone()
            } else {
                "Ready".to_string()
            }
        } else {
            "Ready".to_string()
        };

        let paragraph = Paragraph::new(status_text).style(self.get_normal_style());
        f.render_widget(paragraph, area);
    }

    fn draw_help_overlay(&self, f: &mut Frame, area: Rect) {
        let popup_area = self.centered_rect(70, 80, area);

        f.render_widget(Clear, popup_area);

        let help_text = vec![
            Line::from("LoFi Turtle - Dynamic Layout Demo"),
            Line::from(""),
            Line::from("Playback Controls:"),
            Line::from("  Space    - Play/Pause"),
            Line::from("  +/-      - Volume up/down"),
            Line::from(""),
            Line::from("Navigation:"),
            Line::from("  ↑/↓      - Move selection"),
            Line::from("  Enter    - Select track"),
            Line::from(""),
            Line::from("Layout Controls:"),
            Line::from("  F2       - Switch layout"),
            Line::from("  F3       - Switch theme"),
            Line::from("  F1       - Toggle this help"),
            Line::from(""),
            Line::from("Application:"),
            Line::from("  q/Esc    - Quit"),
            Line::from(""),
            Line::from("Available Layouts:"),
            Line::from("  • Default - Three-panel layout"),
            Line::from("  • Compact - Simple playlist + status"),
            Line::from("  • Minimal - Ultra-simple, no borders"),
        ];

        let paragraph = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title("Help")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(self.get_border_style()),
            )
            .style(self.get_normal_style());

        f.render_widget(paragraph, popup_area);
    }

    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

    fn set_status(&mut self, message: String) {
        self.status = Some((message, Instant::now()));
    }

    // Theme-based styling methods
    fn get_normal_style(&self) -> ratatui::style::Style {
        use ratatui::style::{Color, Style};
        match self.current_theme {
            ThemeType::Dark => Style::default().fg(Color::White),
            ThemeType::Light => Style::default().fg(Color::Black),
            ThemeType::Synthwave => Style::default().fg(Color::Magenta),
            ThemeType::Forest => Style::default().fg(Color::Green),
        }
    }

    fn get_selected_style(&self) -> ratatui::style::Style {
        use ratatui::style::{Color, Style};
        match self.current_theme {
            ThemeType::Dark => Style::default().fg(Color::Yellow),
            ThemeType::Light => Style::default().fg(Color::Blue),
            ThemeType::Synthwave => Style::default().fg(Color::Cyan),
            ThemeType::Forest => Style::default().fg(Color::LightGreen),
        }
    }

    fn get_highlight_style(&self) -> ratatui::style::Style {
        use ratatui::style::{Color, Style};
        match self.current_theme {
            ThemeType::Dark => Style::default().bg(Color::DarkGray),
            ThemeType::Light => Style::default().bg(Color::LightBlue),
            ThemeType::Synthwave => Style::default().bg(Color::DarkGray).fg(Color::Cyan),
            ThemeType::Forest => Style::default().bg(Color::DarkGray),
        }
    }

    fn get_playing_style(&self) -> ratatui::style::Style {
        use ratatui::style::{Color, Style};
        match self.current_theme {
            ThemeType::Dark => Style::default().fg(Color::Green),
            ThemeType::Light => Style::default().fg(Color::Green),
            ThemeType::Synthwave => Style::default().fg(Color::LightCyan),
            ThemeType::Forest => Style::default().fg(Color::LightGreen),
        }
    }

    fn get_paused_style(&self) -> ratatui::style::Style {
        use ratatui::style::{Color, Style};
        match self.current_theme {
            ThemeType::Dark => Style::default().fg(Color::Red),
            ThemeType::Light => Style::default().fg(Color::Red),
            ThemeType::Synthwave => Style::default().fg(Color::LightRed),
            ThemeType::Forest => Style::default().fg(Color::Yellow),
        }
    }

    fn get_secondary_style(&self) -> ratatui::style::Style {
        use ratatui::style::{Color, Style};
        match self.current_theme {
            ThemeType::Dark => Style::default().fg(Color::Gray),
            ThemeType::Light => Style::default().fg(Color::DarkGray),
            ThemeType::Synthwave => Style::default().fg(Color::Blue),
            ThemeType::Forest => Style::default().fg(Color::DarkGray),
        }
    }

    fn get_border_style(&self) -> ratatui::style::Style {
        use ratatui::style::{Color, Style};
        match self.current_theme {
            ThemeType::Dark => Style::default().fg(Color::White),
            ThemeType::Light => Style::default().fg(Color::Black),
            ThemeType::Synthwave => Style::default().fg(Color::Magenta),
            ThemeType::Forest => Style::default().fg(Color::Green),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create and run app
    let mut app = MinimalDemo::new();
    let result = app.run(&mut terminal);

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    match result {
        Ok(_) => println!("Thanks for trying LoFi Turtle Dynamic Layout Demo!"),
        Err(err) => eprintln!("Error: {}", err),
    }

    Ok(())
}
