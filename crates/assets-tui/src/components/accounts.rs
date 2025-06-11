use anyhow::Result;
use ratatui::{
    layout::{Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::component::{Action, Component};
use assets_core::AccountType;
use rust_decimal::Decimal;

pub struct AccountsComponent {
    accounts: Vec<AccountItem>,
    state: ListState,
}

pub struct AccountItem {
    id: String,
    name: String,
    account_type: AccountType,
    balance: Decimal,
}

impl AccountsComponent {
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
            state: ListState::default().with_selected(Some(0)),
        }
    }
    
    // This would connect to the database in a real implementation
    pub async fn load_real_accounts(&mut self) -> Result<()> {
        // In a real implementation, this would connect to the database
        // Example of how it might look:
        // let pool = assets_core::database::connect().await?;
        // let accounts = assets_core::services::AccountService::list_all(&pool).await?;
        // self.accounts = accounts.into_iter().map(|a| AccountItem {
        //     id: a.id.to_string(),
        //     name: a.name,
        //     account_type: a.account_type,
        //     balance: a.balance,
        // }).collect();
        Ok(())
    }
    
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.accounts.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.accounts.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    
    pub fn selected_account(&self) -> Option<&AccountItem> {
        self.state.selected().and_then(|i| self.accounts.get(i))
    }
}

impl Component for AccountsComponent {
    fn init(&mut self) -> Result<()> {
        // Load dummy data synchronously for now
        // In a real implementation, we'd have a separate async function to load from DB
        self.accounts = vec![
            AccountItem {
                id: "1001".to_string(),
                name: "Checking Account".to_string(),
                account_type: AccountType::Asset,
                balance: Decimal::new(1250, 0),
            },
            AccountItem {
                id: "1002".to_string(),
                name: "Savings Account".to_string(),
                account_type: AccountType::Asset,
                balance: Decimal::new(5000, 0),
            },
            AccountItem {
                id: "2001".to_string(),
                name: "Credit Card".to_string(),
                account_type: AccountType::Liability,
                balance: Decimal::new(-500, 0),
            },
        ];
        Ok(())
    }
    
    fn handle_events(&mut self, event: crossterm::event::Event) -> Result<Option<Action>> {
        if let crossterm::event::Event::Key(key) = event {
            if key.kind == crossterm::event::KeyEventKind::Press {
                match key.code {
                    crossterm::event::KeyCode::Down => {
                        self.next();
                    }
                    crossterm::event::KeyCode::Up => {
                        self.previous();
                    }
                    crossterm::event::KeyCode::Enter => {
                        if let Some(account) = self.selected_account() {
                            return Ok(Some(Action::ViewAccount(account.id.clone())));
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(None)
    }
    
    fn render(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        // Create a block for the accounts list
        let block = Block::default()
            .title("Accounts")
            .borders(Borders::ALL);
        
        // Create list items from accounts
        let items: Vec<ListItem> = self.accounts
            .iter()
            .map(|account| {
                let balance_str = format!("€{:.2}", account.balance);
                let line = Span::styled(
                    format!("{} - {}", account.name, balance_str),
                    Style::default().fg(if account.balance < rust_decimal::Decimal::ZERO {
                        Color::Red
                    } else {
                        Color::Green
                    }),
                );
                ListItem::new(line)
            })
            .collect();
        
        // Create the list
        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );
        
        // Render the list
        frame.render_stateful_widget(list, area, &mut self.state.clone());
        
        Ok(())
    }
}
