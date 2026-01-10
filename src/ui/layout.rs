use serde::{Deserialize, Serialize};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use std::collections::HashMap;
use crate::config::layout_config::LayoutConfig;
use crate::error::Result;

/// Position of a component in the layout
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    Left,
    Right,
    Top,
    Bottom,
    Center,
}

/// Size constraint for layout components
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SizeConstraint {
    Percentage(u16),
    Length(u16),
    Min(u16),
    Max(u16),
    Fill,
}

impl From<SizeConstraint> for Constraint {
    fn from(constraint: SizeConstraint) -> Self {
        match constraint {
            SizeConstraint::Percentage(p) => Constraint::Percentage(p),
            SizeConstraint::Length(l) => Constraint::Length(l),
            SizeConstraint::Min(m) => Constraint::Min(m),
            SizeConstraint::Max(m) => Constraint::Max(m),
            SizeConstraint::Fill => Constraint::Fill(1),
        }
    }
}

/// Widget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetConfig {
    pub name: String,
    pub widget_type: WidgetType,
    pub position: Position,
    pub size: SizeConstraint,
    pub visible: bool,
    pub border: bool,
    pub title: Option<String>,
    pub style: WidgetStyle,
}

/// Available widget types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WidgetType {
    Sidebar,
    PlaylistView,
    NowPlaying,
    ProgressBar,
    Waveform,
    StatusBar,
    AlbumArt,
    VolumeControl,
    SearchBox,
}

/// Widget styling configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WidgetStyle {
    pub fg_color: Option<String>,
    pub bg_color: Option<String>,
    pub border_color: Option<String>,
    pub highlight_color: Option<String>,
    pub selected_color: Option<String>,
}

impl Default for WidgetStyle {
    fn default() -> Self {
        Self {
            fg_color: Some("white".to_string()),
            bg_color: Some("black".to_string()),
            border_color: Some("gray".to_string()),
            highlight_color: Some("cyan".to_string()),
            selected_color: Some("yellow".to_string()),
        }
    }
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    pub colors: Option<HashMap<String, String>>,
    pub styles: Option<HashMap<String, StyleConfig>>,
}

/// Style configuration for theme elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConfig {
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
}

/// Border configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorderConfig {
    pub style: String, // "rounded", "plain", "thick", "double"
    pub color: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        let mut colors = HashMap::new();
        colors.insert("primary".to_string(), "cyan".to_string());
        colors.insert("secondary".to_string(), "yellow".to_string());
        colors.insert("background".to_string(), "black".to_string());
        colors.insert("foreground".to_string(), "white".to_string());
        colors.insert("border".to_string(), "gray".to_string());
        colors.insert("highlight".to_string(), "bright_cyan".to_string());
        colors.insert("error".to_string(), "red".to_string());
        colors.insert("success".to_string(), "green".to_string());

        let mut styles = HashMap::new();
        styles.insert("normal".to_string(), StyleConfig {
            fg: Some("foreground".to_string()),
            bg: Some("background".to_string()),
            bold: None,
            italic: None,
            underline: None,
        });
        styles.insert("selected".to_string(), StyleConfig {
            fg: Some("primary".to_string()),
            bg: Some("background".to_string()),
            bold: Some(true),
            italic: None,
            underline: None,
        });

        Self {
            name: "default".to_string(),
            colors: Some(colors),
            styles: Some(styles),
        }
    }
}

// LayoutConfig is now defined in config::layout_config module

/// Layout-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSettings {
    pub auto_save: bool,
    pub debounce_ms: u64,
    pub responsive: ResponsiveBreakpoints,
}

/// Responsive design breakpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsiveBreakpoints {
    pub small_width: u16,   // Terminal width < small
    pub medium_width: u16,  // small <= width < medium
    pub large_width: u16,   // medium <= width < large
}

impl Default for ResponsiveBreakpoints {
    fn default() -> Self {
        Self {
            small_width: 80,
            medium_width: 120,
            large_width: 160,
        }
    }
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self {
            auto_save: true,
            debounce_ms: 300,
            responsive: ResponsiveBreakpoints::default(),
        }
    }
}

// LayoutConfig implementation is now in config::layout_config module

/// Responsive layout modes
#[derive(Debug, Clone, PartialEq)]
pub enum ResponsiveMode {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

/// Layout engine for rendering the UI
pub struct LayoutEngine {
    config: LayoutConfig,
    cached_layouts: HashMap<(u16, u16), Vec<Rect>>, // Cache layouts by terminal size
}

impl LayoutEngine {
    pub fn new(config: LayoutConfig) -> Self {
        Self {
            config,
            cached_layouts: HashMap::new(),
        }
    }

    /// Update the layout configuration
    pub fn update_config(&mut self, config: LayoutConfig) {
        self.config = config;
        self.cached_layouts.clear(); // Clear cache when config changes
    }

    /// Calculate layout for the given terminal area
    #[allow(dead_code)]
    pub fn calculate_layout(&mut self, area: Rect) -> Result<HashMap<String, Rect>> {
        let cache_key = (area.width, area.height);
        
        // Check if we have a cached layout
        if let Some(cached) = self.cached_layouts.get(&cache_key) {
            return Ok(self.map_widgets_to_areas(cached));
        }

        let responsive_mode = self.config.get_responsive_mode(area.width);
        let layout_areas = self.build_layout(area, responsive_mode)?;
        
        // Cache the layout
        self.cached_layouts.insert(cache_key, layout_areas.clone());
        
        Ok(self.map_widgets_to_areas(&layout_areas))
    }

    /// Build the actual layout based on widget configuration
    #[allow(dead_code)]
    fn build_layout(&self, area: Rect, _responsive_mode: ResponsiveMode) -> Result<Vec<Rect>> {
        let visible_widgets: Vec<_> = self.config.widgets
            .iter()
            .filter(|w| w.visible)
            .collect();

        if visible_widgets.is_empty() {
            return Ok(vec![area]);
        }

        // Group widgets by position
        let top_widgets = self.config.widgets_by_position(Position::Top);
        let bottom_widgets = self.config.widgets_by_position(Position::Bottom);
        let left_widgets = self.config.widgets_by_position(Position::Left);
        let right_widgets = self.config.widgets_by_position(Position::Right);
        let center_widgets = self.config.widgets_by_position(Position::Center);

        let mut areas = Vec::new();
        let mut current_area = area;

        // Handle top widgets
        if !top_widgets.is_empty() {
            let constraints: Vec<Constraint> = top_widgets
                .iter()
                .map(|w| w.size.clone().into())
                .collect();
            let top_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(current_area);
            
            areas.extend(top_layout.iter().take(top_widgets.len()).cloned());
            current_area = Rect {
                y: current_area.y + top_layout.iter().take(top_widgets.len()).map(|r| r.height).sum::<u16>(),
                height: current_area.height - top_layout.iter().take(top_widgets.len()).map(|r| r.height).sum::<u16>(),
                ..current_area
            };
        }

        // Handle bottom widgets
        if !bottom_widgets.is_empty() {
            let total_bottom_height: u16 = bottom_widgets
                .iter()
                .map(|w| match &w.size {
                    SizeConstraint::Length(l) => *l,
                    SizeConstraint::Percentage(p) => (area.height * p / 100).max(1),
                    _ => 3, // Default height
                })
                .sum();

            current_area.height = current_area.height.saturating_sub(total_bottom_height);

            let constraints: Vec<Constraint> = bottom_widgets
                .iter()
                .map(|w| w.size.clone().into())
                .collect();
            
            let bottom_area = Rect {
                y: current_area.y + current_area.height,
                height: total_bottom_height,
                ..current_area
            };

            let bottom_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(bottom_area);
            
            areas.extend(bottom_layout.iter().take(bottom_widgets.len()).cloned());
        }

        // Handle horizontal layout (left, center, right)
        let mut horizontal_constraints: Vec<Constraint> = Vec::new();
        let mut horizontal_widget_count = 0;

        if !left_widgets.is_empty() {
            horizontal_constraints.extend(left_widgets.iter().map(|w| -> Constraint { w.size.clone().into() }));
            horizontal_widget_count += left_widgets.len();
        }

        if !center_widgets.is_empty() {
            horizontal_constraints.extend(center_widgets.iter().map(|w| -> Constraint { w.size.clone().into() }));
            horizontal_widget_count += center_widgets.len();
        }

        if !right_widgets.is_empty() {
            horizontal_constraints.extend(right_widgets.iter().map(|w| -> Constraint { w.size.clone().into() }));
            horizontal_widget_count += right_widgets.len();
        }

        if horizontal_widget_count > 0 {
            let horizontal_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(horizontal_constraints)
                .split(current_area);
            
            areas.extend(horizontal_layout.iter().take(horizontal_widget_count).cloned());
        }

        Ok(areas)
    }

    /// Map calculated areas to widget names
    #[allow(dead_code)]
    fn map_widgets_to_areas(&self, areas: &[Rect]) -> HashMap<String, Rect> {
        let mut widget_areas = HashMap::new();
        let visible_widgets: Vec<_> = self.config.widgets
            .iter()
            .filter(|w| w.visible)
            .collect();

        for (i, widget) in visible_widgets.iter().enumerate() {
            if let Some(area) = areas.get(i) {
                widget_areas.insert(widget.name.clone(), *area);
            }
        }

        widget_areas
    }

    /// Get the current layout configuration
    #[allow(dead_code)]
    pub fn config(&self) -> &LayoutConfig {
        &self.config
    }
}
