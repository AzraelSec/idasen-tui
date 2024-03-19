use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::{Block, BorderType, Borders},
    Frame,
};

use crate::{
    app::{
        actions::{AppAction, MovingDirection, SelectionMove},
        app::{App, Views},
        ui_event_emitter::UIEvent,
    },
    tui::help_bar,
};

struct PositionsView;

impl PositionsView {
    fn draw_help_bar(frame: &mut Frame, area: Rect, is_connected: bool) {
        let mut buttons = vec![
            "q: quit".to_string(),
            "e: edit".to_string(),
            "h/j/k/l: move selection".to_string(),
            "arrows: move selection".to_string(),
        ];

        if is_connected {
            buttons.push("<enter>: select".to_string());
        }

        help_bar::draw(frame, area, buttons)
    }
}

impl App {
    pub fn handle_positions_event(&mut self, ev: UIEvent) {
        let state = self.get_current_state();
        match ev {
            UIEvent::KeyPress(ev) => match ev.code {
                KeyCode::Enter => {
                    if let Some(_) = state.connected_device {
                        let current_position = state.positions_list.get_selected();
                        if let Some(height) = current_position {
                            self.start_action(AppAction::StartMoving(MovingDirection::ToHeight(
                                height.height,
                            )))
                        }
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.move_position_selection(SelectionMove::Prev)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.move_position_selection(SelectionMove::Next)
                }
                _ => (),
            },
            _ => (),
        }
    }

    pub fn draw_positions(&mut self, frame: &mut Frame, view_area: Rect, bar_area: Rect) {
        let mut state = self.get_current_state();

        let mut container = Block::default()
            .borders(Borders::ALL)
            .title("Saved positions")
            .border_type(BorderType::Rounded);

        if state.current_view == Views::SavedPositions {
            container = container.border_style(Style::new().bold().light_red());
        }

        frame.render_widget(container.clone(), view_area);
        state
            .positions_list
            .draw(frame, container.inner(view_area), |_| false);

        if state.current_view == Views::SavedPositions {
            PositionsView::draw_help_bar(
                frame,
                bar_area,
                if let Some(_) = state.connected_device {
                    true
                } else {
                    false
                },
            )
        }
    }
}
