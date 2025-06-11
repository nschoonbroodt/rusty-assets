use anyhow::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, List, ListItem, ListState},
    Frame,
};

use crate::component::{Action, Component};

pub struct SettingsComponent {
    settings: Vec<Setting>,
    state: ListState,
}

struct Setting {
    name: String,
    value: String,
    description: String,
}

impl SettingsComponent {
    pub fn new() -> Self {
        Self {
            settings: vec![
                Setting {
                    name: "Database Connection".to_string(),
                    value: "PostgreSQL".to_string(),
                    description: "Configure database connection settings".to_string(),
                },
                Setting {
                    name: "Currency".to_string(),
                    value: "EUR".to_string(),
                    description: "Set the default currency for the application".to_string(),
                },
                Setting {
                    name: "Theme".to_string(),
                    value: "Default".to_string(),
                    description: "Change the application theme".to_string(),
                },
                Setting {
                    name: "Date Format".to_string(),
                    value: "YYYY-MM-DD".to_string(),
                    description: "Set the default date format".to_string(),
                },
            ],
            state: ListState::default().with_selected(Some(0)),
        }
    }
    
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.settings.len() - 1 {
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
                    self.settings.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    
    pub fn selected_setting(&self) -> Option<&Setting> {
        self.state.selected().and_then(|i| self.settings.get(i))
    }
}

impl Component for SettingsComponent {
    fn init(&mut self) -> Result<()> {
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
                    _ => {}
                }
            }
        }
        Ok(None)
    }
    
    fn render(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        // Create layout for settings list and description
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
            .split(area);
        
        // Create settings list
        let items: Vec<ListItem> = self.settings
            .iter()
            .map(|setting| {
                ListItem::new(format!("{}: {}", setting.name, setting.value))
            })
            .collect();
        
        let list = List::new(items)
            .block(Block::default().title("Settings").borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );
        
        frame.render_stateful_widget(list, chunks[0], &mut self.state.clone());
        
        // Display description of selected setting
        if let Some(setting) = self.selected_setting() {
            let description = Paragraph::new(setting.description.clone())
                .block(Block::default().title("Description").borders(Borders::ALL))
                .style(Style::default().fg(Color::White));
            
            frame.render_widget(description, chunks[1]);
        }
        
        Ok(())
    }
}
