use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};

use crate::component::{Action, App, Component};
use crate::ui::UI;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    pub app: App,
    pub ui: UI,
    pub tick_rate: Duration,
    pub last_tick: Instant,
}

impl Tui {    pub async fn new() -> Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        
        // Create app
        let app = App::new();
        
        // Initialize app with database connection
        app.init().await?;

        Ok(Self {
            terminal,
            app,
            ui: UI::new(),
            tick_rate: Duration::from_millis(250),
            last_tick: Instant::now(),
        })
    }

    pub async fn init(&mut self) -> Result<()> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        // Initialize UI components with database
        self.ui.init_with_db(self.app.db.clone()).await?;

        // Hide cursor
        self.terminal.hide_cursor()?;

        // Clear the screen
        self.terminal.clear()?;

        Ok(())
    }    pub async fn run(&mut self) -> Result<()> {
        self.init().await?;

        // Main loop
        while !self.app.should_quit {
            self.draw()?;
            self.handle_events().await?;
            self.tick()?;
        }

        self.exit()?;

        Ok(())
    }

    pub fn draw(&mut self) -> Result<()> {
        self.terminal.draw(|frame| {
            // Let the UI component handle rendering
            if let Err(e) = self.ui.render(frame, frame.size()) {
                // Log error
                eprintln!("Error rendering UI: {}", e);
            }
        })?;

        Ok(())
    }    pub async fn handle_events(&mut self) -> Result<()> {
        // Poll for events with a timeout
        if crossterm::event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    // Handle global key events
                    match key.code {
                        KeyCode::Char('q') => {
                            self.app.handle_action(Action::Quit)?;
                            return Ok(());
                        }                        
                        _ => {
                            // Pass event to UI for component-specific handling
                            if let Some(action) = self.ui.handle_events(Event::Key(key))? {
                                match &action {
                                    Action::Refresh => {
                                        // Reload data from database
                                        self.ui.refresh_data().await?;
                                    }
                                    _ => {}
                                }
                                
                                self.app.handle_action(action)?;
                            }
                        }
                    }
                }                Event::Resize(_, _) => {
                    // Handle terminal resize
                    self.terminal.clear()?;
                }
                event => {
                    // Pass other events to UI
                    if let Some(action) = self.ui.handle_events(event)? {
                        self.app.handle_action(action)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn tick(&mut self) -> Result<()> {
        let now = Instant::now();
        if now.duration_since(self.last_tick) >= self.tick_rate {
            self.last_tick = now;
            self.app.tick()?;
        }

        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        // Restore terminal
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;

        Ok(())
    }
}
