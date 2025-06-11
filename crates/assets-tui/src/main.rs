use anyhow::{Context, Result};
use std::error::Error;
use std::panic;

mod component;
mod components;
mod tui;
mod ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Set up custom panic hook to properly restore terminal state
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Attempt to restore terminal state
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        );
        
        // Call the default panic handler
        default_hook(panic_info);
    }));

    // Initialize the TUI
    let mut tui = tui::Tui::new()
        .context("Failed to initialize TUI")?;
    
    // Run the application
    tui.run()
        .context("Error while running the application")?;
    
    Ok(())
}
