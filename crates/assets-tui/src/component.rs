use anyhow::Result;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub trait Component {
    /// Initialize the component
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    /// Handle events for the component
    fn handle_events(&mut self, event: crossterm::event::Event) -> Result<Option<Action>> {
        Ok(None)
    }

    /// Update the component state
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }

    /// Render the component
    fn render(&self, frame: &mut Frame, area: Rect) -> Result<()>;
}

/// Actions that can be performed on components
#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    Navigate(Screen),
    ViewAccount(String),
    ViewTransaction(String),
    AddAccount,
    AddTransaction,
    EditAccount(String),
    EditTransaction(String),
    DeleteAccount(String),
    DeleteTransaction(String),
    Refresh,
    Help,
    Error(String),
}

/// The different screens in the application
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Accounts,
    Transactions,
    Reports,
    Settings,
}

/// The main application state
pub struct App {
    pub should_quit: bool,
    pub current_screen: Screen,
    pub db: crate::database::AppDatabase,
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_quit: false,
            current_screen: Screen::Accounts,
            db: crate::database::AppDatabase::new(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub async fn init(&self) -> anyhow::Result<()> {
        // Initialize database connection
        self.db.init().await?;
        Ok(())
    }
    
    pub fn tick(&self) -> anyhow::Result<()> {
        Ok(())
    }
    pub fn handle_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => {
                self.should_quit = true;
            }
            Action::Navigate(screen) => {
                self.current_screen = screen;
            }
            Action::ViewAccount(_)
            | Action::ViewTransaction(_)
            | Action::AddAccount
            | Action::AddTransaction
            | Action::EditAccount(_)
            | Action::EditTransaction(_)
            | Action::DeleteAccount(_)
            | Action::DeleteTransaction(_)
            | Action::Refresh
            | Action::Help => {
                // Handle specific actions
                // For now, we're just ignoring them
            }
            Action::Error(msg) => {
                eprintln!("Error: {}", msg);
            }
        }
        Ok(())
    }

    pub async fn refresh_data(&self) -> Result<()> {
        // Refresh data from database
        // This will be implemented by UI to reload data from components
        Ok(())
    }
}
