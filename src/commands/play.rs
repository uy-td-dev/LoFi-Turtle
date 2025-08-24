use crate::commands::Command;
use crate::config::Config;
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
        log::info!("Starting interactive music player...");
        
        // Initialize and run the TUI service
        let mut tui_service = TuiService::new(config)?;
        tui_service.run()
    }

    fn description(&self) -> &'static str {
        "Start the interactive music player interface"
    }
}
