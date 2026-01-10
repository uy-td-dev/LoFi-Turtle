//! Layout configuration structures and parsing logic
//! 
//! This module defines the core layout configuration structures and provides
//! functionality for parsing TOML configuration files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use crate::error::{LofiTurtleError, Result};
use crate::ui::layout::{WidgetConfig, LayoutSettings};
use crate::ui::layout::ThemeConfig;

/// Complete layout configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    /// Configuration version for compatibility checking
    pub version: String,
    
    /// Human-readable name for this layout
    pub name: String,
    
    /// Optional description of the layout
    pub description: Option<String>,
    
    /// Theme configuration
    pub theme: ThemeConfig,
    
    /// Widget configurations
    pub widgets: Vec<WidgetConfig>,
    
    /// Key bindings mapping
    pub keybindings: HashMap<String, String>,
    
    /// Layout settings and preferences
    pub settings: LayoutSettings,
}

impl LayoutConfig {
    /// Load layout configuration from TOML file
    /// Returns error if file cannot be read or parsed
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| LofiTurtleError::Configuration(
                format!("Failed to read layout config from {}: {}", path.display(), e)
            ))?;
        
        Self::parse_from_string(&content)
    }
    
    /// Parse layout configuration from TOML string
    pub fn parse_from_string(content: &str) -> Result<Self> {
        toml::from_str(content)
            .map_err(|e| LofiTurtleError::Configuration(
                format!("Failed to parse layout config: {}", e)
            ))
    }
    
    /// Parse partial layout configuration from TOML string
    /// This allows for incomplete configs that will be merged with defaults
    #[allow(dead_code)]
    pub fn parse_partial_from_string(content: &str) -> Result<toml::Value> {
        toml::from_str(content)
            .map_err(|e| LofiTurtleError::Configuration(
                format!("Failed to parse partial layout config: {}", e)
            ))
    }
    
    /// Save layout configuration to TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let content = toml::to_string_pretty(self)
            .map_err(|e| LofiTurtleError::Configuration(
                format!("Failed to serialize layout config: {}", e)
            ))?;

        std::fs::write(path, content)
            .map_err(|e| LofiTurtleError::Configuration(
                format!("Failed to write layout config to {}: {}", path.display(), e)
            ))?;
        
        Ok(())
    }
    
    /// Convert to TOML string
    pub fn to_toml_string(&self) -> Result<String> {
        toml::to_string_pretty(self)
            .map_err(|e| LofiTurtleError::Configuration(
                format!("Failed to serialize layout config: {}", e)
            ))
    }
    
    /// Validate the layout configuration
    pub fn validate(&self) -> Result<()> {
        // Check version compatibility
        if self.version.is_empty() {
            return Err(LofiTurtleError::Configuration(
                "Layout version cannot be empty".to_string()
            ));
        }
        
        // Check that we have at least one visible widget
        if !self.widgets.iter().any(|w| w.visible) {
            return Err(LofiTurtleError::Configuration(
                "At least one widget must be visible".to_string()
            ));
        }
        
        // Check for duplicate widget names
        let mut widget_names = std::collections::HashSet::new();
        for widget in &self.widgets {
            if !widget_names.insert(&widget.name) {
                return Err(LofiTurtleError::Configuration(
                    format!("Duplicate widget name: {}", widget.name)
                ));
            }
        }
        
        // Validate keybindings (check for empty keys or actions)
        for (key, action) in &self.keybindings {
            if key.is_empty() {
                return Err(LofiTurtleError::Configuration(
                    "Keybinding key cannot be empty".to_string()
                ));
            }
            if action.is_empty() {
                return Err(LofiTurtleError::Configuration(
                    format!("Keybinding action for key '{}' cannot be empty", key)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Get widget by name
    pub fn get_widget(&self, name: &str) -> Option<&WidgetConfig> {
        self.widgets.iter().find(|w| w.name == name)
    }
    
    /// Get mutable widget by name
    #[allow(dead_code)]
    pub fn get_widget_mut(&mut self, name: &str) -> Option<&mut WidgetConfig> {
        self.widgets.iter_mut().find(|w| w.name == name)
    }
    
    /// Check if a widget exists and is visible
    pub fn is_widget_visible(&self, name: &str) -> bool {
        self.get_widget(name).map_or(false, |w| w.visible)
    }
    
    /// Get all visible widgets
    pub fn get_visible_widgets(&self) -> Vec<&WidgetConfig> {
        self.widgets.iter().filter(|w| w.visible).collect()
    }
    
    /// Get keybinding action for a key
    pub fn get_keybinding(&self, key: &str) -> Option<&String> {
        self.keybindings.get(key)
    }
    
    /// Add or update a keybinding
    pub fn set_keybinding(&mut self, key: String, action: String) {
        self.keybindings.insert(key, action);
    }
    
    /// Remove a keybinding
    pub fn remove_keybinding(&mut self, key: &str) -> Option<String> {
        self.keybindings.remove(key)
    }
    
    /// Get widgets by position
    #[allow(dead_code)]
    pub fn widgets_by_position(&self, position: crate::ui::layout::Position) -> Vec<&WidgetConfig> {
        self.widgets
            .iter()
            .filter(|w| w.visible && w.position == position)
            .collect()
    }

    /// Check if layout should adapt for terminal size
    pub fn get_responsive_mode(&self, terminal_width: u16) -> crate::ui::layout::ResponsiveMode {
        let breakpoints = &self.settings.responsive;
        if terminal_width < breakpoints.small_width {
            crate::ui::layout::ResponsiveMode::Small
        } else if terminal_width < breakpoints.medium_width {
            crate::ui::layout::ResponsiveMode::Medium
        } else if terminal_width < breakpoints.large_width {
            crate::ui::layout::ResponsiveMode::Large
        } else {
            crate::ui::layout::ResponsiveMode::ExtraLarge
        }
    }
}

impl Default for LayoutConfig {
    fn default() -> Self {
        crate::config::defaults::create_default_layout()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_config_validation() {
        let mut config = LayoutConfig::default();
        
        // Valid config should pass
        assert!(config.validate().is_ok());
        
        // Empty version should fail
        config.version = String::new();
        assert!(config.validate().is_err());
        
        // Reset version
        config.version = "1.0".to_string();
        
        // No visible widgets should fail
        for widget in &mut config.widgets {
            widget.visible = false;
        }
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_duplicate_widget_names() {
        let mut config = LayoutConfig::default();
        let widget = config.widgets[0].clone();
        config.widgets.push(widget); // Add duplicate

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_widget_operations() {
        let config = LayoutConfig::default();
        
        // Test getting widget
        assert!(config.get_widget("sidebar").is_some());
        assert!(config.get_widget("nonexistent").is_none());
        
        // Test widget visibility
        assert!(config.is_widget_visible("sidebar"));
        
        // Test getting visible widgets
        let visible_widgets = config.get_visible_widgets();
        assert!(!visible_widgets.is_empty());
    }
    
    #[test]
    fn test_keybinding_operations() {
        let mut config = LayoutConfig::default();
        
        // Test getting keybinding
        assert_eq!(config.get_keybinding("space"), Some(&"toggle_play".to_string()));
        assert_eq!(config.get_keybinding("nonexistent"), None);
        
        // Test setting keybinding
        config.set_keybinding("test".to_string(), "test_action".to_string());
        assert_eq!(config.get_keybinding("test"), Some(&"test_action".to_string()));
        
        // Test removing keybinding
        let removed = config.remove_keybinding("test");
        assert_eq!(removed, Some("test_action".to_string()));
        assert_eq!(config.get_keybinding("test"), None);
    }
    
    #[test]
    fn test_toml_serialization() {
        let config = LayoutConfig::default();
        
        // Test serialization
        let toml_string = config.to_toml_string().unwrap();
        assert!(!toml_string.is_empty());
        
        // Test deserialization
        let parsed_config = LayoutConfig::parse_from_string(&toml_string).unwrap();
        assert_eq!(config.name, parsed_config.name);
        assert_eq!(config.version, parsed_config.version);
    }

    #[test]
    fn test_responsive_mode() {
        let config = LayoutConfig::default();
        // Default breakpoints: small=80, medium=120, large=160

        assert_eq!(config.get_responsive_mode(79), crate::ui::layout::ResponsiveMode::Small);
        assert_eq!(config.get_responsive_mode(80), crate::ui::layout::ResponsiveMode::Medium);
        assert_eq!(config.get_responsive_mode(119), crate::ui::layout::ResponsiveMode::Medium);
        assert_eq!(config.get_responsive_mode(120), crate::ui::layout::ResponsiveMode::Large);
        assert_eq!(config.get_responsive_mode(160), crate::ui::layout::ResponsiveMode::ExtraLarge);
    }
}
