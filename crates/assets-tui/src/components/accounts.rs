use anyhow::Result;
use ratatui::{
    layout::{Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::component::{Action, Component};
use crate::database::AppDatabase;
use assets_core::AccountType;
use rust_decimal::Decimal;
use uuid::Uuid;

pub struct AccountsComponent {
    accounts: Vec<AccountItem>,
    state: ListState,
    db: Option<AppDatabase>,
}

pub struct AccountItem {
    pub id: String,
    pub name: String,
    pub account_type: AccountType,
    pub balance: Decimal,
}

impl AccountsComponent {
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
            state: ListState::default().with_selected(Some(0)),
            db: None,
        }
    }
    
    pub fn set_accounts(&mut self, accounts: Vec<AccountItem>) {
        self.accounts = accounts;
        // Reset selection if needed
        if !self.accounts.is_empty() {
            self.state = ListState::default().with_selected(Some(0));
        }
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
    
    pub fn init_with_db(&mut self, db: AppDatabase) -> Result<()> {
        self.db = Some(db);
        Ok(())
    }
      pub async fn load_accounts(&mut self) -> Result<()> {
        if let Some(db) = &self.db {
            let db_conn = db.get().await?;
            let pool = db_conn.pool().clone();
            
            // Use the AccountService to get account summaries with balances
            let account_service = assets_core::services::AccountService::new(pool.clone());
            let account_summaries = account_service.get_account_summaries().await?;
            
            // Convert to AccountItems and update the component
            let mut account_items = Vec::with_capacity(account_summaries.len());
            
            for summary in account_summaries {
                account_items.push(AccountItem {
                    id: summary.id.to_string(),
                    name: summary.name,
                    account_type: summary.account_type,
                    balance: summary.balance,
                });
            }
            
            // Update the accounts
            self.set_accounts(account_items);
        } else {
            // Fallback to dummy data if no database is available
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
        }
        
        Ok(())
    }
}

impl Component for AccountsComponent {
    fn init(&mut self) -> Result<()> {
        // We'll load the data in an async context later
        // This is just initial setup for non-async parts
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
