use serde::{Deserialize, Serialize};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use std::collections::HashMap;
use crate::config::layout_config::LayoutConfig;
use crate::error::Result;

/// Position of a component in the layout
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    Left,
    Right,
    Top,
    Bottom,
    Center,
}

/// Size constraint for layout components
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

    #[serde(rename = "type", alias = "widget_type")]
    pub widget_type: WidgetType,

    pub position: Position,

    pub size: SizeConstraint,

    #[serde(default = "default_true")]
    pub visible: bool,

    #[serde(default)]
    pub border: bool,

    #[serde(default)]
    pub title: Option<String>,

    #[serde(default)]
    pub style: WidgetStyle,
}

fn default_true() -> bool {
    true
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
            fg_color: None,
            bg_color: None,
            border_color: None,
            highlight_color: None,
            selected_color: None,
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

impl Default for ThemeConfig {
    fn default() -> Self {
        let mut colors = HashMap::new();
        colors.insert("primary".to_string(), "cyan".to_string());
        colors.insert("background".to_string(), "black".to_string());
        colors.insert("foreground".to_string(), "white".to_string());

        Self {
            name: "default".to_string(),
            colors: Some(colors),
            styles: None,
        }
    }
}

/// Layout-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSettings {
    #[serde(default = "default_true")]
    pub auto_save: bool,

    #[serde(default = "default_debounce")]
    pub debounce_ms: u64,

    #[serde(default)]
    pub responsive: ResponsiveBreakpoints,
}

fn default_debounce() -> u64 {
    300
}

/// Responsive design breakpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsiveBreakpoints {
    pub small_width: u16,
    pub medium_width: u16,
    pub large_width: u16,
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
    cached_layouts: HashMap<(u16, u16), HashMap<String, Rect>>,
}

impl LayoutEngine {
    pub fn new(config: LayoutConfig) -> Self {
        Self {
            config,
            cached_layouts: HashMap::new(),
        }
    }

    pub fn update_config(&mut self, config: LayoutConfig) {
        self.config = config;
        self.cached_layouts.clear();
    }

    /// Calculate layout for the given terminal area
    pub fn calculate_layout(&mut self, area: Rect) -> Result<HashMap<String, Rect>> {
        let cache_key = (area.width, area.height);
        
        if let Some(cached) = self.cached_layouts.get(&cache_key) {
            return Ok(cached.clone());
        }

        let responsive_mode = self.config.get_responsive_mode(area.width);
        let layout_map = self.build_layout(area, responsive_mode)?;
        
        self.cached_layouts.insert(cache_key, layout_map.clone());
        
        Ok(layout_map)
    }

    /// Build the actual layout based on widget configuration
    /// Refactored to return a Map directly, ensuring widget names match their areas
    fn build_layout(&self, area: Rect, _responsive_mode: ResponsiveMode) -> Result<HashMap<String, Rect>> {
        let mut result = HashMap::new();
        let mut current_area = area;

        // 1. Process Top Widgets
        let top_widgets = self.get_visible_widgets_by_pos(Position::Top);
        if !top_widgets.is_empty() {
            let (areas, remaining) = self.split_vertical(current_area, &top_widgets, true);
            for (widget, rect) in top_widgets.iter().zip(areas.into_iter()) {
                result.insert(widget.name.clone(), rect);
            }
            current_area = remaining;
        }

        // 2. Process Bottom Widgets
        let bottom_widgets = self.get_visible_widgets_by_pos(Position::Bottom);
        if !bottom_widgets.is_empty() {
            let (areas, remaining) = self.split_vertical(current_area, &bottom_widgets, false);
            for (widget, rect) in bottom_widgets.iter().zip(areas.into_iter()) {
                result.insert(widget.name.clone(), rect);
            }
            current_area = remaining;
        }

        // 3. Process Middle (Left, Center, Right)
        let left_widgets = self.get_visible_widgets_by_pos(Position::Left);
        let center_widgets = self.get_visible_widgets_by_pos(Position::Center);
        let right_widgets = self.get_visible_widgets_by_pos(Position::Right);

        let mut constraints: Vec<Constraint> = Vec::new();
        let mut middle_widgets = Vec::new();

        // Collect all horizontal widgets in order: Left -> Center -> Right
        for w in &left_widgets {
            constraints.push(w.size.clone().into());
            middle_widgets.push(w);
        }
        for w in &center_widgets {
            constraints.push(w.size.clone().into());
            middle_widgets.push(w);
        }
        for w in &right_widgets {
            constraints.push(w.size.clone().into());
            middle_widgets.push(w);
        }

        if !constraints.is_empty() {
            let horizontal_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(current_area);

            for (widget, rect) in middle_widgets.iter().zip(horizontal_layout.iter()) {
                result.insert(widget.name.clone(), *rect);
            }
        }

        Ok(result)
    }

    /// Helper to get visible widgets for a specific position
    fn get_visible_widgets_by_pos(&self, pos: Position) -> Vec<&WidgetConfig> {
        self.config.widgets.iter()
            .filter(|w| w.visible && w.position == pos)
            .collect()
    }

    /// Helper to split an area vertically (for Top/Bottom)
    /// Returns (Vector of Rects for widgets, Remaining Rect)
    fn split_vertical(&self, area: Rect, widgets: &[&WidgetConfig], is_top: bool) -> (Vec<Rect>, Rect) {
        if is_top {
            let mut constraints: Vec<Constraint> = widgets.iter()
                .map(|w| w.size.clone().into())
                .collect();

            // Add a constraint for the remaining space
            constraints.push(Constraint::Fill(1));

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area);

            let widget_areas = layout.iter().take(widgets.len()).cloned().collect();
            let remaining = *layout.last().unwrap_or(&area);

            (widget_areas, remaining)
        } else {
            // Bottom logic
            // We want the widgets to be at the bottom.
            // Layout: [Remaining (Fill), Widget 1, Widget 2...]

            let mut bottom_constraints = vec![Constraint::Fill(1)]; // Top filler
            bottom_constraints.extend(widgets.iter().map(|w| -> Constraint { w.size.clone().into() }));

            let bottom_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(bottom_constraints)
                .split(area);
            
            let remaining = bottom_layout[0];
            let widget_areas = bottom_layout.iter().skip(1).cloned().collect();

            (widget_areas, remaining)
        }
    }

    pub fn config(&self) -> &LayoutConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::layout_config::LayoutConfig;

    fn create_test_config(widgets: Vec<WidgetConfig>) -> LayoutConfig {
        let mut config = LayoutConfig::default();
        config.widgets = widgets;
        config
    }

    fn create_widget(name: &str, pos: Position, size: SizeConstraint) -> WidgetConfig {
        WidgetConfig {
            name: name.to_string(),
            widget_type: WidgetType::Sidebar, // Dummy type
            position: pos,
            size,
            visible: true,
            border: false,
            title: None,
            style: WidgetStyle::default(),
        }
    }

    #[test]
    fn test_layout_top_only() {
        let widgets = vec![
            create_widget("top1", Position::Top, SizeConstraint::Length(10)),
        ];
        let config = create_test_config(widgets);
        let mut engine = LayoutEngine::new(config);
        let area = Rect::new(0, 0, 100, 100);

        let result = engine.calculate_layout(area).unwrap();

        assert!(result.contains_key("top1"));
        let rect = result.get("top1").unwrap();
        assert_eq!(rect.height, 10);
        assert_eq!(rect.y, 0);
    }

    #[test]
    fn test_layout_bottom_only() {
        let widgets = vec![
            create_widget("bot1", Position::Bottom, SizeConstraint::Length(10)),
        ];
        let config = create_test_config(widgets);
        let mut engine = LayoutEngine::new(config);
        let area = Rect::new(0, 0, 100, 100);

        let result = engine.calculate_layout(area).unwrap();

        assert!(result.contains_key("bot1"));
        let rect = result.get("bot1").unwrap();
        assert_eq!(rect.height, 10);
        assert_eq!(rect.y, 90); // 100 - 10
    }

    #[test]
    fn test_layout_top_and_bottom() {
        let widgets = vec![
            create_widget("top", Position::Top, SizeConstraint::Length(10)),
            create_widget("bot", Position::Bottom, SizeConstraint::Length(10)),
        ];
        let config = create_test_config(widgets);
        let mut engine = LayoutEngine::new(config);
        let area = Rect::new(0, 0, 100, 100);

        let result = engine.calculate_layout(area).unwrap();

        let top = result.get("top").unwrap();
        assert_eq!(top.y, 0);
        assert_eq!(top.height, 10);

        let bot = result.get("bot").unwrap();
        assert_eq!(bot.y, 90);
        assert_eq!(bot.height, 10);
    }

    #[test]
    fn test_layout_middle_split() {
        let widgets = vec![
            create_widget("left", Position::Left, SizeConstraint::Percentage(20)),
            create_widget("center", Position::Center, SizeConstraint::Fill),
            create_widget("right", Position::Right, SizeConstraint::Percentage(20)),
        ];
        let config = create_test_config(widgets);
        let mut engine = LayoutEngine::new(config);
        let area = Rect::new(0, 0, 100, 100);

        let result = engine.calculate_layout(area).unwrap();

        let left = result.get("left").unwrap();
        assert_eq!(left.x, 0);
        assert_eq!(left.width, 20);

        let right = result.get("right").unwrap();
        // Ratatui layout calculation might vary slightly with Fill, but roughly:
        // Left 20, Center Fill, Right 20.
        // Center should be ~60.
        assert!(right.x >= 80);
        assert_eq!(right.width, 20);

        let center = result.get("center").unwrap();
        assert_eq!(center.x, 20);
        assert_eq!(center.width, 60);
    }

    #[test]
    fn test_complex_layout() {
        // Top (10), Bottom (10), Left (20%), Center (Fill)
        let widgets = vec![
            create_widget("top", Position::Top, SizeConstraint::Length(10)),
            create_widget("bot", Position::Bottom, SizeConstraint::Length(10)),
            create_widget("left", Position::Left, SizeConstraint::Percentage(20)),
            create_widget("center", Position::Center, SizeConstraint::Fill),
        ];
        let config = create_test_config(widgets);
        let mut engine = LayoutEngine::new(config);
        let area = Rect::new(0, 0, 100, 100);

        let result = engine.calculate_layout(area).unwrap();

        // Check vertical space remaining for middle: 100 - 10 - 10 = 80
        let left = result.get("left").unwrap();
        assert_eq!(left.height, 80);
        assert_eq!(left.y, 10); // Starts after top

        let center = result.get("center").unwrap();
        assert_eq!(center.height, 80);
    }

    #[test]
    fn test_hidden_widget() {
        let mut w = create_widget("hidden", Position::Top, SizeConstraint::Length(10));
        w.visible = false;
        let widgets = vec![w];
        let config = create_test_config(widgets);
        let mut engine = LayoutEngine::new(config);
        let area = Rect::new(0, 0, 100, 100);

        let result = engine.calculate_layout(area).unwrap();
        assert!(result.is_empty());
    }
}
