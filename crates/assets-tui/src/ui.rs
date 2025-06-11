use anyhow::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::component::{Action, Component, Screen};
use crate::components::{
    accounts::AccountsComponent,
    reports::ReportsComponent,
    settings::SettingsComponent,
    transactions::TransactionsComponent,
};

pub struct UI {
    accounts: AccountsComponent,
    transactions: TransactionsComponent,
    reports: ReportsComponent,
    settings: SettingsComponent,
    current_screen: Screen,
}

impl UI {
    pub fn new() -> Self {
        Self {
            accounts: AccountsComponent::new(),
            transactions: TransactionsComponent::new(),
            reports: ReportsComponent::new(),
            settings: SettingsComponent::new(),
            current_screen: Screen::Accounts,
        }
    }
    
    pub fn init(&mut self) -> Result<()> {
        self.accounts.init()?;
        self.transactions.init()?;
        self.reports.init()?;
        self.settings.init()?;
        Ok(())
    }
    
    pub fn set_screen(&mut self, screen: Screen) {
        self.current_screen = screen;
    }
}

impl Component for UI {
    fn handle_events(&mut self, event: crossterm::event::Event) -> Result<Option<Action>> {
        // Handle tab navigation with numbers
        if let crossterm::event::Event::Key(key) = &event {
            if key.kind == crossterm::event::KeyEventKind::Press {
                match key.code {
                    crossterm::event::KeyCode::Char('1') => {
                        self.current_screen = Screen::Accounts;
                        return Ok(Some(Action::Navigate(Screen::Accounts)));
                    }
                    crossterm::event::KeyCode::Char('2') => {
                        self.current_screen = Screen::Transactions;
                        return Ok(Some(Action::Navigate(Screen::Transactions)));
                    }
                    crossterm::event::KeyCode::Char('3') => {
                        self.current_screen = Screen::Reports;
                        return Ok(Some(Action::Navigate(Screen::Reports)));
                    }
                    crossterm::event::KeyCode::Char('4') => {
                        self.current_screen = Screen::Settings;
                        return Ok(Some(Action::Navigate(Screen::Settings)));
                    }
                    _ => {}
                }
            }
        }
        
        // Pass events to the active component
        match self.current_screen {
            Screen::Accounts => self.accounts.handle_events(event),
            Screen::Transactions => self.transactions.handle_events(event),
            Screen::Reports => self.reports.handle_events(event),
            Screen::Settings => self.settings.handle_events(event),
        }
    }
      fn render(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        // Create the layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)].as_ref())
            .split(area);
        
        // Render tabs
        let _ = self.render_tabs(frame, chunks[0]);
        
        // Render the active component
        match self.current_screen {
            Screen::Accounts => self.accounts.render(frame, chunks[1])?,
            Screen::Transactions => self.transactions.render(frame, chunks[1])?,
            Screen::Reports => self.reports.render(frame, chunks[1])?,
            Screen::Settings => self.settings.render(frame, chunks[1])?,
        }
        
        // Render help text at bottom
        let help_text = match self.current_screen {
            Screen::Accounts => "↑↓: Navigate  Enter: View details  q: Quit",
            Screen::Transactions => "↑↓: Navigate  Enter: View details  q: Quit",
            Screen::Reports => "←→: Change report  q: Quit",
            Screen::Settings => "↑↓: Navigate  q: Quit",
        };
        
        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help_paragraph, chunks[2]);
        
        Ok(())
    }
}

impl UI {
    fn render_tabs(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        let titles = vec![
            "1:Accounts",
            "2:Transactions",
            "3:Reports",
            "4:Settings",
        ];
        
        let tab_index = match self.current_screen {
            Screen::Accounts => 0,
            Screen::Transactions => 1,
            Screen::Reports => 2,
            Screen::Settings => 3,
        };
        
        let tabs = Tabs::new(
            titles.into_iter().map(Line::from).collect(),
        )
        .block(Block::default().borders(Borders::ALL).title("RustyAssets TUI"))
        .select(tab_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        );
        
        frame.render_widget(tabs, area);
        Ok(())
    }
}
