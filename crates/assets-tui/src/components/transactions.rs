use anyhow::Result;
use ratatui::{
    layout::{Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::component::{Action, Component};
use crate::database::AppDatabase;
use assets_core::services::TransactionService;
use rust_decimal::Decimal;
use uuid::Uuid;

pub struct TransactionsComponent {
    transactions: Vec<TransactionItem>,
    state: ListState,
    db: Option<AppDatabase>,
}

pub struct TransactionItem {
    id: String,
    date: String,
    description: String,
    amount: Decimal,
    account: String,
}

impl TransactionsComponent {
    pub fn new() -> Self {
        Self {
            transactions: Vec::new(),
            state: ListState::default().with_selected(Some(0)),
            db: None,
        }
    }
    
    pub fn init_with_db(&mut self, db: AppDatabase) -> Result<()> {
        self.db = Some(db);
        Ok(())
    }    pub async fn load_transactions(&mut self) -> Result<()> {
        if let Some(db) = &self.db {
            let db_conn = db.get().await?;
            let pool = db_conn.pool().clone();
            
            // Use the TransactionService to get recent transactions with account information
            let transaction_service = TransactionService::new(pool.clone());
            
            // Get the 30 most recent transactions from the database with account info
            let transactions = transaction_service.get_recent_transactions_with_accounts(30).await?;
            
            // Convert to TransactionItems
            self.transactions.clear();
            
            for transaction in transactions {
                self.transactions.push(TransactionItem {
                    id: transaction.id.to_string(),
                    date: transaction.transaction_date.format("%Y-%m-%d").to_string(),
                    description: transaction.description,
                    amount: transaction.amount,
                    account: transaction.account_name,
                });
            }
            
            // Reset selection
            if !self.transactions.is_empty() {
                self.state = ListState::default().with_selected(Some(0));
            }
        } else {
            // Fallback to dummy data if no database is available
            self.transactions = vec![
                TransactionItem {
                    id: Uuid::new_v4().to_string(),
                    date: "2025-06-01".to_string(),
                    description: "Grocery Shopping".to_string(),
                    amount: Decimal::new(-7850, 2),
                    account: "Checking Account".to_string(),
                },
                TransactionItem {
                    id: Uuid::new_v4().to_string(),
                    date: "2025-06-05".to_string(),
                    description: "Salary".to_string(),
                    amount: Decimal::new(250000, 2),
                    account: "Checking Account".to_string(),
                },
                TransactionItem {
                    id: Uuid::new_v4().to_string(),
                    date: "2025-06-07".to_string(),
                    description: "Utilities".to_string(),
                    amount: Decimal::new(-12500, 2),
                    account: "Credit Card".to_string(),
                },
            ];
        }
        
        Ok(())
    }
    
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.transactions.len() - 1 {
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
                    self.transactions.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    
    pub fn selected_transaction(&self) -> Option<&TransactionItem> {
        self.state.selected().and_then(|i| self.transactions.get(i))
    }
}

impl Component for TransactionsComponent {
    fn init(&mut self) -> Result<()> {
        // We'll load data asynchronously later
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
                        if let Some(transaction) = self.selected_transaction() {
                            return Ok(Some(Action::ViewTransaction(transaction.id.clone())));
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(None)
    }
    
    fn render(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        // Create a block for the transactions list
        let block = Block::default()
            .title("Transactions")
            .borders(Borders::ALL);
        
        if self.transactions.is_empty() {
            let paragraph = Paragraph::new("No transactions found")
                .block(block);
            frame.render_widget(paragraph, area);
            return Ok(());
        }
        
        // Create list items from transactions
        let items: Vec<ListItem> = self.transactions
            .iter()
            .map(|transaction| {
                let amount_str = format!("€{:.2}", transaction.amount);
                let line = format!(
                    "{} | {} | {} | {}", 
                    transaction.date,
                    transaction.description,
                    amount_str,
                    transaction.account
                );
                
                let style = if transaction.amount < rust_decimal::Decimal::ZERO {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::Green)
                };
                
                ListItem::new(line).style(style)
            })
            .collect();            // Create the list
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
