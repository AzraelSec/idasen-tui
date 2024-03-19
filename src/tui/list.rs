use std::ops::Index;

use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style, Stylize},
    widgets::{List, ListItem, ListState, Paragraph},
    Frame,
};

pub trait ListableItem {
    fn render_row(&self) -> String;
    fn is_highlighted(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct StatefulList<T: ListableItem> {
    items: Vec<T>,
    state: ListState,
}

impl<T: ListableItem> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> Self {
        let state = if items.is_empty() {
            ListState::default()
        } else {
            ListState::default().with_selected(Some(0))
        };

        Self { items, state }
    }

    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            state: ListState::default(),
        }
    }

    pub fn select_prev(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(idx) => {
                if idx == 0 {
                    self.items.len() - 1
                } else {
                    idx - 1
                }
            }
            None => self.items.len() - 1,
        };

        self.state.select(Some(i))
    }

    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(idx) => {
                if idx >= self.items.len() - 1 {
                    0
                } else {
                    idx + 1
                }
            }
            None => 0,
        };

        self.state.select(Some(i))
    }

    pub fn get_selected(&self) -> Option<&T> {
        if let Some(idx) = self.state.selected() {
            self.items.get(idx)
        } else {
            None
        }
    }

    pub fn is_current_selected_highlighted(&self) -> bool {
        if let Some(idx) = self.state.selected() {
            self.items.index(idx).is_highlighted()
        } else {
            false
        }
    }

    pub fn set(&mut self, items: Vec<T>) {
        self.items = items;
        if self.items.is_empty() {
            self.state.select(None)
        } else {
            self.state.select(Some(0))
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect, is_highlighted: impl Fn(&T) -> bool) {
        if self.items.is_empty() {
            frame.render_widget(
                Paragraph::new("no items").alignment(Alignment::Center),
                area,
            )
        } else {
            let item_style = Style::default();
            let highlight_style = item_style.bold().black().on_green();

            let items: Vec<ListItem> = self
                .items
                .iter()
                .map(|i| {
                    let row_str = i.render_row();
                    ListItem::new(row_str).style(if is_highlighted(i) {
                        highlight_style
                    } else {
                        item_style
                    })
                })
                .collect();
            let list = List::new(items)
                // .block(
                //     Block::default()
                //         .borders(Borders::ALL)
                //         .border_type(BorderType::Rounded),
                // )
                .highlight_style(item_style.add_modifier(Modifier::BOLD | Modifier::REVERSED))
                .highlight_symbol("> ");

            frame.render_stateful_widget(list, area, &mut self.state)
        }
    }
}
