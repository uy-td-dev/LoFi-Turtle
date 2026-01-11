use ratatui::style::{Color, Style, Modifier};
use std::collections::HashMap;
use crate::ui::layout::ThemeConfig;

/// Color palette for the application
#[derive(Debug, Clone)]
pub struct ColorPalette {
    colors: HashMap<String, Color>,
}

impl ColorPalette {
    /// Create a new color palette from theme config
    pub fn from_theme(theme: &ThemeConfig) -> Self {
        let mut colors = HashMap::new();
        
        if let Some(ref theme_colors) = theme.colors {
            for (name, color_str) in theme_colors {
                if let Some(color) = Self::parse_color(color_str) {
                    colors.insert(name.clone(), color);
                } else {
                    log::warn!("Invalid color '{}' for theme color '{}'", color_str, name);
                }
            }
        }
        
        Self { colors }
    }

    /// Get a color by name
    #[allow(dead_code)]
    pub fn get(&self, name: &str) -> Option<Color> {
        self.colors.get(name).copied()
    }

    /// Get a color by name with fallback
    pub fn get_or(&self, name: &str, fallback: Color) -> Color {
        self.colors.get(name).copied().unwrap_or(fallback)
    }

    /// Parse color string to ratatui Color
    pub fn parse_color(color_str: &str) -> Option<Color> {
        match color_str.to_lowercase().as_str() {
            "black" => Some(Color::Black),
            "red" => Some(Color::Red),
            "green" => Some(Color::Green),
            "yellow" => Some(Color::Yellow),
            "blue" => Some(Color::Blue),
            "magenta" => Some(Color::Magenta),
            "cyan" => Some(Color::Cyan),
            "gray" | "grey" => Some(Color::Gray),
            "dark_gray" | "dark_grey" => Some(Color::DarkGray),
            "light_red" | "bright_red" => Some(Color::LightRed),
            "light_green" | "bright_green" => Some(Color::LightGreen),
            "light_yellow" | "bright_yellow" => Some(Color::LightYellow),
            "light_blue" | "bright_blue" => Some(Color::LightBlue),
            "light_magenta" | "bright_magenta" => Some(Color::LightMagenta),
            "light_cyan" | "bright_cyan" => Some(Color::LightCyan),
            "white" => Some(Color::White),
            "reset" => Some(Color::Reset),
            _ => {
                // Try to parse as RGB hex color
                if color_str.starts_with('#') {
                    let hex = if color_str.len() == 7 {
                        &color_str[1..]
                    } else if color_str.len() == 4 {
                        // Handle short hex #RGB -> #RRGGBB (not implemented here for simplicity, but good to know)
                        // For now just standard 6-digit hex
                        return None;
                    } else {
                        return None;
                    };

                    if let Ok(rgb) = u32::from_str_radix(hex, 16) {
                        let r = ((rgb >> 16) & 0xFF) as u8;
                        let g = ((rgb >> 8) & 0xFF) as u8;
                        let b = (rgb & 0xFF) as u8;
                        return Some(Color::Rgb(r, g, b));
                    }
                }
                // Try to parse as indexed color
                if let Ok(index) = color_str.parse::<u8>() {
                    return Some(Color::Indexed(index));
                }
                None
            }
        }
    }

    /// Helper to get color from Option<String> or fallback
    pub fn get_color_from_option(color_str: Option<&String>, fallback: Color) -> Color {
        if let Some(s) = color_str {
            Self::parse_color(s).unwrap_or(fallback)
        } else {
            fallback
        }
    }
}

/// Theme manager for styling UI components
#[derive(Debug, Clone)]
pub struct ThemeManager {
    palette: ColorPalette,
    #[allow(dead_code)]
    theme_config: ThemeConfig,
}

impl ThemeManager {
    /// Create a new theme manager
    #[allow(dead_code)]
    pub fn new(theme_config: ThemeConfig) -> Self {
        let palette = ColorPalette::from_theme(&theme_config);
        Self {
            palette,
            theme_config,
        }
    }

    /// Get the color palette
    #[allow(dead_code)]
    pub fn palette(&self) -> &ColorPalette {
        &self.palette
    }

    /// Get the theme configuration
    #[allow(dead_code)]
    pub fn config(&self) -> &ThemeConfig {
        &self.theme_config
    }

    /// Create a style for normal text
    #[allow(dead_code)]
    pub fn normal_style(&self) -> Style {
        Style::default()
            .fg(self.palette.get_or("foreground", Color::White))
            .bg(self.palette.get_or("background", Color::Black))
    }

    /// Create a style for highlighted text
    #[allow(dead_code)]
    pub fn highlight_style(&self) -> Style {
        Style::default()
            .fg(self.palette.get_or("highlight", Color::Cyan))
            .bg(self.palette.get_or("background", Color::Black))
            .add_modifier(Modifier::BOLD)
    }

    /// Create a style for selected items
    #[allow(dead_code)]
    pub fn selected_style(&self) -> Style {
        Style::default()
            .fg(self.palette.get_or("background", Color::Black))
            .bg(self.palette.get_or("primary", Color::Cyan))
            .add_modifier(Modifier::BOLD)
    }

    /// Create a style for borders
    #[allow(dead_code)]
    pub fn border_style(&self) -> Style {
        Style::default()
            .fg(self.palette.get_or("border", Color::Gray))
    }

    /// Create a style for titles
    #[allow(dead_code)]
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.palette.get_or("primary", Color::Cyan))
            .add_modifier(Modifier::BOLD)
    }

    /// Create a style for playing track
    #[allow(dead_code)]
    pub fn playing_style(&self) -> Style {
        Style::default()
            .fg(self.palette.get_or("playing", Color::Green))
            .add_modifier(Modifier::BOLD)
    }

    /// Create a style for paused track
    #[allow(dead_code)]
    pub fn paused_style(&self) -> Style {
        Style::default()
            .fg(self.palette.get_or("paused", Color::Yellow))
    }

    /// Create a style for progress bar
    #[allow(dead_code)]
    pub fn progress_style(&self) -> Style {
        Style::default()
            .fg(self.palette.get_or("progress", Color::Blue))
            .bg(self.palette.get_or("background", Color::Black))
    }

    /// Create a style for error messages
    #[allow(dead_code)]
    pub fn error_style(&self) -> Style {
        Style::default()
            .fg(self.palette.get_or("error", Color::Red))
            .add_modifier(Modifier::BOLD)
    }

    /// Create a style for success messages
    #[allow(dead_code)]
    pub fn success_style(&self) -> Style {
        Style::default()
            .fg(self.palette.get_or("success", Color::Green))
            .add_modifier(Modifier::BOLD)
    }

    /// Create a style for secondary text
    #[allow(dead_code)]
    pub fn secondary_style(&self) -> Style {
        Style::default()
            .fg(self.palette.get_or("secondary", Color::Yellow))
    }

    /// Create a custom style with specific colors
    #[allow(dead_code)]
    pub fn custom_style(&self, fg: Option<&str>, bg: Option<&str>) -> Style {
        let mut style = Style::default();
        
        if let Some(fg_name) = fg {
            if let Some(color) = self.palette.get(fg_name) {
                style = style.fg(color);
            }
        }
        
        if let Some(bg_name) = bg {
            if let Some(color) = self.palette.get(bg_name) {
                style = style.bg(color);
            }
        }
        
        style
    }

    /// Update theme configuration
    #[allow(dead_code)]
    pub fn update_theme(&mut self, theme_config: ThemeConfig) {
        self.theme_config = theme_config;
        self.palette = ColorPalette::from_theme(&self.theme_config);
    }
}

/// Predefined themes
pub struct Themes;

impl Themes {
    /// Default dark theme
    pub fn dark() -> ThemeConfig {
        let mut colors = HashMap::new();
        colors.insert("primary".to_string(), "cyan".to_string());
        colors.insert("secondary".to_string(), "yellow".to_string());
        colors.insert("background".to_string(), "black".to_string());
        colors.insert("foreground".to_string(), "white".to_string());
        colors.insert("border".to_string(), "gray".to_string());
        colors.insert("highlight".to_string(), "bright_cyan".to_string());
        colors.insert("error".to_string(), "red".to_string());
        colors.insert("success".to_string(), "green".to_string());
        colors.insert("playing".to_string(), "bright_green".to_string());
        colors.insert("paused".to_string(), "bright_yellow".to_string());
        colors.insert("progress".to_string(), "blue".to_string());

        ThemeConfig {
            name: "dark".to_string(),
            colors: Some(colors),
            styles: None,
        }
    }

    /// Light theme
    pub fn light() -> ThemeConfig {
        let mut colors = HashMap::new();
        colors.insert("primary".to_string(), "blue".to_string());
        colors.insert("secondary".to_string(), "magenta".to_string());
        colors.insert("background".to_string(), "white".to_string());
        colors.insert("foreground".to_string(), "black".to_string());
        colors.insert("border".to_string(), "dark_gray".to_string());
        colors.insert("highlight".to_string(), "bright_blue".to_string());
        colors.insert("error".to_string(), "red".to_string());
        colors.insert("success".to_string(), "green".to_string());
        colors.insert("playing".to_string(), "green".to_string());
        colors.insert("paused".to_string(), "yellow".to_string());
        colors.insert("progress".to_string(), "blue".to_string());

        ThemeConfig {
            name: "light".to_string(),
            colors: Some(colors),
            styles: None,
        }
    }

    /// Retro/synthwave theme
    pub fn synthwave() -> ThemeConfig {
        let mut colors = HashMap::new();
        colors.insert("primary".to_string(), "#ff00ff".to_string()); // Magenta
        colors.insert("secondary".to_string(), "#00ffff".to_string()); // Cyan
        colors.insert("background".to_string(), "#0a0a0a".to_string()); // Very dark
        colors.insert("foreground".to_string(), "#ffffff".to_string());
        colors.insert("border".to_string(), "#ff00ff".to_string());
        colors.insert("highlight".to_string(), "#ffff00".to_string()); // Yellow
        colors.insert("error".to_string(), "#ff0080".to_string());
        colors.insert("success".to_string(), "#00ff80".to_string());
        colors.insert("playing".to_string(), "#00ff00".to_string());
        colors.insert("paused".to_string(), "#ffff00".to_string());
        colors.insert("progress".to_string(), "#8000ff".to_string());

        ThemeConfig {
            name: "synthwave".to_string(),
            colors: Some(colors),
            styles: None,
        }
    }

    /// Forest/nature theme
    pub fn forest() -> ThemeConfig {
        let mut colors = HashMap::new();
        colors.insert("primary".to_string(), "#228b22".to_string()); // Forest green
        colors.insert("secondary".to_string(), "#daa520".to_string()); // Goldenrod
        colors.insert("background".to_string(), "#0f1419".to_string()); // Dark forest
        colors.insert("foreground".to_string(), "#e6e6e6".to_string());
        colors.insert("border".to_string(), "#556b2f".to_string()); // Dark olive green
        colors.insert("highlight".to_string(), "#32cd32".to_string()); // Lime green
        colors.insert("error".to_string(), "#dc143c".to_string());
        colors.insert("success".to_string(), "#90ee90".to_string());
        colors.insert("playing".to_string(), "#00ff7f".to_string());
        colors.insert("paused".to_string(), "#ffd700".to_string());
        colors.insert("progress".to_string(), "#4682b4".to_string());

        ThemeConfig {
            name: "forest".to_string(),
            colors: Some(colors),
            styles: None,
        }
    }

    /// Dracula theme
    pub fn dracula() -> ThemeConfig {
        let mut colors = HashMap::new();
        colors.insert("primary".to_string(), "#bd93f9".to_string()); // Purple
        colors.insert("secondary".to_string(), "#ff79c6".to_string()); // Pink
        colors.insert("background".to_string(), "#282a36".to_string()); // Dark
        colors.insert("foreground".to_string(), "#f8f8f2".to_string()); // White-ish
        colors.insert("border".to_string(), "#6272a4".to_string()); // Blue-gray
        colors.insert("highlight".to_string(), "#8be9fd".to_string()); // Cyan
        colors.insert("error".to_string(), "#ff5555".to_string()); // Red
        colors.insert("success".to_string(), "#50fa7b".to_string()); // Green
        colors.insert("playing".to_string(), "#50fa7b".to_string()); // Green
        colors.insert("paused".to_string(), "#ffb86c".to_string()); // Orange
        colors.insert("progress".to_string(), "#bd93f9".to_string()); // Purple

        ThemeConfig {
            name: "dracula".to_string(),
            colors: Some(colors),
            styles: None,
        }
    }

    /// Gruvbox theme
    pub fn gruvbox() -> ThemeConfig {
        let mut colors = HashMap::new();
        colors.insert("primary".to_string(), "#fabd2f".to_string()); // Yellow
        colors.insert("secondary".to_string(), "#fe8019".to_string()); // Orange
        colors.insert("background".to_string(), "#282828".to_string()); // Dark
        colors.insert("foreground".to_string(), "#ebdbb2".to_string()); // Light
        colors.insert("border".to_string(), "#928374".to_string()); // Gray
        colors.insert("highlight".to_string(), "#83a598".to_string()); // Blue
        colors.insert("error".to_string(), "#cc241d".to_string()); // Red
        colors.insert("success".to_string(), "#b8bb26".to_string()); // Green
        colors.insert("playing".to_string(), "#b8bb26".to_string()); // Green
        colors.insert("paused".to_string(), "#fabd2f".to_string()); // Yellow
        colors.insert("progress".to_string(), "#d3869b".to_string()); // Purple

        ThemeConfig {
            name: "gruvbox".to_string(),
            colors: Some(colors),
            styles: None,
        }
    }

    /// Get all available themes
    pub fn all() -> Vec<ThemeConfig> {
        vec![
            Self::dark(),
            Self::light(),
            Self::synthwave(),
            Self::forest(),
            Self::dracula(),
            Self::gruvbox(),
        ]
    }

    /// Get theme by name
    #[allow(dead_code)]
    pub fn get_by_name(name: &str) -> Option<ThemeConfig> {
        match name.to_lowercase().as_str() {
            "dark" => Some(Self::dark()),
            "light" => Some(Self::light()),
            "synthwave" => Some(Self::synthwave()),
            "forest" => Some(Self::forest()),
            "dracula" => Some(Self::dracula()),
            "gruvbox" => Some(Self::gruvbox()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_parsing() {
        assert_eq!(ColorPalette::parse_color("red"), Some(Color::Red));
        assert_eq!(ColorPalette::parse_color("bright_cyan"), Some(Color::LightCyan));
        assert_eq!(ColorPalette::parse_color("#ff0000"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(ColorPalette::parse_color("42"), Some(Color::Indexed(42)));
        assert_eq!(ColorPalette::parse_color("invalid"), None);
    }

    #[test]
    fn test_theme_manager() {
        let theme = Themes::dark();
        let manager = ThemeManager::new(theme);
        
        let normal = manager.normal_style();
        assert_eq!(normal.fg, Some(Color::White));
        assert_eq!(normal.bg, Some(Color::Black));
        
        let highlight = manager.highlight_style();
        assert_eq!(highlight.fg, Some(Color::LightCyan));
        assert!(highlight.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_predefined_themes() {
        let themes = Themes::all();
        assert_eq!(themes.len(), 6);
        
        let dark = Themes::get_by_name("dark").unwrap();
        assert_eq!(dark.name, "dark");
        
        let synthwave = Themes::get_by_name("synthwave").unwrap();
        assert_eq!(synthwave.name, "synthwave");

        let dracula = Themes::get_by_name("dracula").unwrap();
        assert_eq!(dracula.name, "dracula");
    }
}
