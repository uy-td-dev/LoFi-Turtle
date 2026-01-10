//! Default layout configuration constants and functions
//! 
//! This module provides the default layout configuration that serves as a fallback
//! when user configuration is missing or incomplete.

use std::collections::HashMap;
use crate::config::layout_config::LayoutConfig;
use crate::ui::layout::{WidgetConfig, WidgetType, Position, SizeConstraint, WidgetStyle, LayoutSettings, ResponsiveBreakpoints};

/// Default layout configuration as a TOML string
/// This is used when no user configuration file is found
#[allow(dead_code)]
pub fn get_default_layout_toml() -> String {
    format!(r##"version = "1.0"
name = "Lofi Night"
description = "A chill, dark theme with rounded borders and vibrant accents."

[theme]
name = "lofi_night"

[theme.colors]
primary = "#bd93f9"      # Dracula Purple
secondary = "#ff79c6"    # Dracula Pink
background = "#282a36"   # Dracula Background
foreground = "#f8f8f2"   # Dracula Foreground
border = "#6272a4"       # Dracula Comment/Selection
highlight = "#8be9fd"    # Dracula Cyan
error = "#ff5555"        # Dracula Red
success = "#50fa7b"      # Dracula Green

[[widgets]]
name = "sidebar"
type = "sidebar"
position = "left"
size = {{ percentage = 25 }}
visible = true
border = true
title = "Library"

[[widgets]]
name = "playlist"
type = "playlist_view"
position = "center"
size = "fill"
visible = true
border = true
title = "Current Playlist"

[[widgets]]
name = "now_playing"
type = "now_playing"
position = "right"
size = {{ percentage = 30 }}
visible = true
border = true
title = "Now Playing"

[[widgets]]
name = "album_art"
type = "album_art"
position = "right"
size = {{ percentage = 25 }}
visible = true
border = true
title = "Visuals"

[[widgets]]
name = "progress"
type = "progress_bar"
position = "bottom"
size = {{ length = 3 }}
visible = true
border = false

[[widgets]]
name = "status"
type = "status_bar"
position = "bottom"
size = {{ length = 1 }}
visible = true
border = false

[keybindings]
space = "toggle_play"
n = "next_track"
p = "previous_track"
up = "move_up"
down = "move_down"
enter = "select"
"+" = "volume_up"
"-" = "volume_down"
f1 = "help"
f2 = "switch_layout"
f3 = "switch_theme"
f5 = "reload_layout"
q = "quit"
esc = "quit"
"/" = "search"
"a" = "toggle_art"

[settings]
auto_save = true
debounce_ms = 300

[settings.responsive]
small_width = 80
medium_width = 120
large_width = 160"##)
}

/// Create default layout configuration
pub fn create_default_layout() -> LayoutConfig {
    let mut keybindings = HashMap::new();
    keybindings.insert("space".to_string(), "toggle_play".to_string());
    keybindings.insert("n".to_string(), "next_track".to_string());
    keybindings.insert("p".to_string(), "previous_track".to_string());
    keybindings.insert("up".to_string(), "move_up".to_string());
    keybindings.insert("down".to_string(), "move_down".to_string());
    keybindings.insert("enter".to_string(), "select".to_string());
    keybindings.insert("+".to_string(), "volume_up".to_string());
    keybindings.insert("-".to_string(), "volume_down".to_string());
    keybindings.insert("f1".to_string(), "help".to_string());
    keybindings.insert("f2".to_string(), "switch_layout".to_string());
    keybindings.insert("f3".to_string(), "switch_theme".to_string());
    keybindings.insert("f5".to_string(), "reload_layout".to_string());
    keybindings.insert("q".to_string(), "quit".to_string());
    keybindings.insert("esc".to_string(), "quit".to_string());
    keybindings.insert("/".to_string(), "search".to_string());
    keybindings.insert("a".to_string(), "toggle_art".to_string());

    // Create custom theme config
    let mut colors = HashMap::new();
    colors.insert("primary".to_string(), "#bd93f9".to_string());
    colors.insert("secondary".to_string(), "#ff79c6".to_string());
    colors.insert("background".to_string(), "#282a36".to_string());
    colors.insert("foreground".to_string(), "#f8f8f2".to_string());
    colors.insert("border".to_string(), "#6272a4".to_string());
    colors.insert("highlight".to_string(), "#8be9fd".to_string());
    colors.insert("error".to_string(), "#ff5555".to_string());
    colors.insert("success".to_string(), "#50fa7b".to_string());

    let theme = crate::ui::layout::ThemeConfig {
        name: "lofi_night".to_string(),
        colors: Some(colors),
        styles: None,
    };

    LayoutConfig {
        version: "1.0".to_string(),
        name: "Lofi Night".to_string(),
        description: Some("A chill, dark theme with rounded borders and vibrant accents.".to_string()),
        theme,
        widgets: vec![
            WidgetConfig {
                name: "sidebar".to_string(),
                widget_type: WidgetType::Sidebar,
                position: Position::Left,
                size: SizeConstraint::Percentage(25),
                visible: true,
                border: true,
                title: Some("Library".to_string()),
                style: WidgetStyle::default(),
            },
            WidgetConfig {
                name: "playlist".to_string(),
                widget_type: WidgetType::PlaylistView,
                position: Position::Center,
                size: SizeConstraint::Fill,
                visible: true,
                border: true,
                title: Some("Current Playlist".to_string()),
                style: WidgetStyle::default(),
            },
            WidgetConfig {
                name: "now_playing".to_string(),
                widget_type: WidgetType::NowPlaying,
                position: Position::Right,
                size: SizeConstraint::Percentage(30),
                visible: true,
                border: true,
                title: Some("Now Playing".to_string()),
                style: WidgetStyle::default(),
            },
            WidgetConfig {
                name: "album_art".to_string(),
                widget_type: WidgetType::AlbumArt,
                position: Position::Right,
                size: SizeConstraint::Percentage(25),
                visible: true,
                border: true,
                title: Some("Visuals".to_string()),
                style: WidgetStyle::default(),
            },
            WidgetConfig {
                name: "progress".to_string(),
                widget_type: WidgetType::ProgressBar,
                position: Position::Bottom,
                size: SizeConstraint::Length(3),
                visible: true,
                border: false,
                title: None,
                style: WidgetStyle::default(),
            },
            WidgetConfig {
                name: "status".to_string(),
                widget_type: WidgetType::StatusBar,
                position: Position::Bottom,
                size: SizeConstraint::Length(1),
                visible: true,
                border: false,
                title: None,
                style: WidgetStyle::default(),
            },
        ],
        keybindings,
        settings: LayoutSettings {
            auto_save: true,
            debounce_ms: 300,
            responsive: ResponsiveBreakpoints {
                small_width: 80,
                medium_width: 120,
                large_width: 160,
            },
        },
    }
}

/// Get default keybindings
#[allow(dead_code)]
pub fn get_default_keybindings() -> HashMap<String, String> {
    let mut keybindings = HashMap::new();
    keybindings.insert("space".to_string(), "toggle_play".to_string());
    keybindings.insert("n".to_string(), "next_track".to_string());
    keybindings.insert("p".to_string(), "previous_track".to_string());
    keybindings.insert("up".to_string(), "move_up".to_string());
    keybindings.insert("down".to_string(), "move_down".to_string());
    keybindings.insert("enter".to_string(), "select".to_string());
    keybindings.insert("+".to_string(), "volume_up".to_string());
    keybindings.insert("-".to_string(), "volume_down".to_string());
    keybindings.insert("f1".to_string(), "help".to_string());
    keybindings.insert("f2".to_string(), "switch_layout".to_string());
    keybindings.insert("f3".to_string(), "switch_theme".to_string());
    keybindings.insert("f5".to_string(), "reload_layout".to_string());
    keybindings.insert("q".to_string(), "quit".to_string());
    keybindings.insert("esc".to_string(), "quit".to_string());
    keybindings.insert("/".to_string(), "search".to_string());
    keybindings.insert("a".to_string(), "toggle_art".to_string());
    keybindings
}

/// Get default widgets configuration
#[allow(dead_code)]
pub fn get_default_widgets() -> Vec<WidgetConfig> {
    vec![
        WidgetConfig {
            name: "sidebar".to_string(),
            widget_type: WidgetType::Sidebar,
            position: Position::Left,
            size: SizeConstraint::Percentage(25),
            visible: true,
            border: true,
            title: Some("Library".to_string()),
            style: WidgetStyle::default(),
        },
        WidgetConfig {
            name: "playlist".to_string(),
            widget_type: WidgetType::PlaylistView,
            position: Position::Center,
            size: SizeConstraint::Fill,
            visible: true,
            border: true,
            title: Some("Current Playlist".to_string()),
            style: WidgetStyle::default(),
        },
        WidgetConfig {
            name: "now_playing".to_string(),
            widget_type: WidgetType::NowPlaying,
            position: Position::Right,
            size: SizeConstraint::Percentage(30),
            visible: true,
            border: true,
            title: Some("Now Playing".to_string()),
            style: WidgetStyle::default(),
        },
        WidgetConfig {
            name: "album_art".to_string(),
            widget_type: WidgetType::AlbumArt,
            position: Position::Right,
            size: SizeConstraint::Percentage(25),
            visible: true,
            border: true,
            title: Some("Visuals".to_string()),
            style: WidgetStyle::default(),
        },
        WidgetConfig {
            name: "progress".to_string(),
            widget_type: WidgetType::ProgressBar,
            position: Position::Bottom,
            size: SizeConstraint::Length(3),
            visible: true,
            border: false,
            title: None,
            style: WidgetStyle::default(),
        },
        WidgetConfig {
            name: "status".to_string(),
            widget_type: WidgetType::StatusBar,
            position: Position::Bottom,
            size: SizeConstraint::Length(1),
            visible: true,
            border: false,
            title: None,
            style: WidgetStyle::default(),
        },
    ]
}

/// Get default layout settings
#[allow(dead_code)]
pub fn get_default_settings() -> LayoutSettings {
    LayoutSettings {
        auto_save: true,
        debounce_ms: 300,
        responsive: ResponsiveBreakpoints {
            small_width: 80,
            medium_width: 120,
            large_width: 160,
        },
    }
}
