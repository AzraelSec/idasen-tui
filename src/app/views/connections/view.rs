use btleplug::api::BDAddr;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph},
    Frame,
};

use crate::{
    app::app::{App, Views},
    tui::help_bar,
};

struct ConnectionsView;

impl ConnectionsView {
    fn draw_help_bar(frame: &mut Frame, area: Rect) {
        help_bar::draw(frame, area, vec!["q: quit".to_string()])
    }

    fn draw_connection_box(
        frame: &mut Frame,
        area: Rect,
        is_focused: bool,
        connected_device: Option<BDAddr>,
    ) {
        let mut container = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .padding(Padding::new(4, 4, 2, 2))
            .title("Connection");

        if is_focused {
            container = container.border_style(Style::new().bold().light_red());
        }

        let mut content = vec![
            Line::from("Welcome to idasen-tui!"),
            Line::from(""),
            if let Some(addr) = connected_device {
                Line::from(vec![
                    "Connected Device: ".into(),
                    addr.to_string().green(),
                ])
            } else {
                Line::from(Span::from("Disconnected").red())
            },
        ];

        if let None = connected_device {
            content.append(&mut vec![
                Line::from(""),
                Line::from("Select your Idåsen device using the Devices section"),
            ])
        }

        frame.render_widget(Paragraph::new(content).block(container), area);
    }
}

impl App {
    pub fn draw_connections(&mut self, frame: &mut Frame, view_area: Rect, bar_area: Rect) {
        let state = self.get_current_state();
        ConnectionsView::draw_connection_box(
            frame,
            view_area,
            state.current_view == Views::Connection,
            state.connected_device,
        );

        if state.current_view == Views::Connection {
            ConnectionsView::draw_help_bar(frame, bar_area)
        }
    }
}
