use anyhow::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::component::{Action, Component};

pub struct ReportsComponent {
    current_tab: usize,
    report_tabs: Vec<&'static str>,
}

impl ReportsComponent {
    pub fn new() -> Self {
        Self {
            current_tab: 0,
            report_tabs: vec!["Monthly Summary", "Income vs Expenses", "Net Worth"],
        }
    }
    
    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % self.report_tabs.len();
    }
    
    pub fn previous_tab(&mut self) {
        if self.current_tab > 0 {
            self.current_tab -= 1;
        } else {
            self.current_tab = self.report_tabs.len() - 1;
        }
    }
}

impl Component for ReportsComponent {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }
    
    fn handle_events(&mut self, event: crossterm::event::Event) -> Result<Option<Action>> {
        if let crossterm::event::Event::Key(key) = event {
            if key.kind == crossterm::event::KeyEventKind::Press {
                match key.code {
                    crossterm::event::KeyCode::Right => {
                        self.next_tab();
                    }
                    crossterm::event::KeyCode::Left => {
                        self.previous_tab();
                    }
                    _ => {}
                }
            }
        }
        Ok(None)
    }
    
    fn render(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        // Create layout for tabs and content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);
        
        // Create the tabs
        let tabs = Tabs::new(self.report_tabs.iter().map(|t| t.to_string()).collect())
            .block(Block::default().title("Reports").borders(Borders::ALL))
            .select(self.current_tab)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        
        frame.render_widget(tabs, chunks[0]);
        
        // Render the content for the selected tab
        let content_block = Block::default()
            .borders(Borders::ALL)
            .title(self.report_tabs[self.current_tab]);
        
        let report_content = match self.current_tab {
            0 => "Monthly Summary Report\n\nIncome: €2,500.00\nExpenses: €1,850.00\nNet: €650.00",
            1 => "Income vs Expenses Report\n\nThis would show a graph of income vs expenses over time.",
            2 => "Net Worth Report\n\nAssets: €6,250.00\nLiabilities: €500.00\nNet Worth: €5,750.00",
            _ => "Report not available",
        };
        
        let paragraph = Paragraph::new(report_content)
            .block(content_block)
            .style(Style::default().fg(Color::White));
        
        frame.render_widget(paragraph, chunks[1]);
        
        Ok(())
    }
}
