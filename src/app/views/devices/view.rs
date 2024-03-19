use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::{Block, BorderType, Borders},
    Frame,
};

use crate::{
    app::{
        actions::{AppAction, SelectionMove},
        app::{App, Views},
        state::Status,
        ui_event_emitter::UIEvent,
    },
    tui::help_bar,
};

struct DevicesListView;

impl DevicesListView {
    fn draw_help_bar(frame: &mut Frame, area: Rect, is_connected: bool) {
        let mut buttons = vec![
            "q: quit".to_string(),
            "r: refresh".to_string(),
            "h/j/k/l: move selection".to_string(),
            "arrows: move selection".to_string(),
        ];

        if !is_connected {
            buttons.push("<enter>: connect".to_string());
        }

        help_bar::draw(frame, area, buttons)
    }
}

impl App {
    pub fn handle_device_list_event(&mut self, ev: UIEvent) {
        let state = self.get_current_state();
        match ev {
            UIEvent::KeyPress(ev) => match ev.code {
                KeyCode::Char('r') => {
                    if let Status::Running = state.status {
                        self.start_action(AppAction::UpdateDevicesList)
                    }
                }
                KeyCode::Enter => self.connect_selected(),
                KeyCode::Up | KeyCode::Char('k') => self.move_device_selection(SelectionMove::Prev),
                KeyCode::Down | KeyCode::Char('j') => {
                    self.move_device_selection(SelectionMove::Next)
                }
                _ => (),
            },
            _ => (),
        }
    }

    pub fn draw_device_list(&mut self, frame: &mut Frame, view_area: Rect, bar_area: Rect) {
        let mut state = self.get_current_state();

        let mut container = Block::default()
            .title("Devices")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        if state.current_view == Views::DeviceList {
            container = container.border_style(Style::new().bold().light_red());
        }

        frame.render_widget(container.clone(), view_area);

        let is_highlighted = state.devices_list.is_current_selected_highlighted();
        state
            .devices_list
            .draw(frame, container.inner(view_area), |f| {
                match state.connected_device {
                    None => false,
                    Some(d) => f.properties.address == d,
                }
            });

        if state.current_view == Views::DeviceList {
            DevicesListView::draw_help_bar(frame, bar_area, is_highlighted);
        }
    }
}
