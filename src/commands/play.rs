use crate::commands::Command;
use crate::config::{Config, LayoutConfig};
use crate::error::Result;
use crate::services::TuiService;

/// Command to start the interactive music player
pub struct PlayCommand;

impl PlayCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for PlayCommand {
    fn execute(&self, config: &Config) -> Result<()> {
        // Use default layout config for backward compatibility
        self.execute_with_layout(config, &LayoutConfig::default())
    }
    
    fn execute_with_layout(&self, config: &Config, layout_config: &LayoutConfig) -> Result<()> {
        log::info!("Starting interactive music player with layout: {}", layout_config.name);
        
        // Initialize and run the TUI service with layout config
        let mut tui_service = TuiService::new(config, layout_config)?;
        tui_service.run()
    }

    fn description(&self) -> &'static str {
        "Start the interactive music player interface"
    }
}
