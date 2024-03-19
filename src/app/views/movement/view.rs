use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Borders, Padding},
    Frame,
};
use tui_big_text::{BigText, PixelSize};

use crate::{
    app::{
        actions::MovingDirection,
        app::{App, Views},
        ui_event_emitter::UIEvent,
    },
    idasen::idasen::{Direction, PositionSpeed},
    tui::help_bar,
};

struct MovementView;

impl MovementView {
    fn draw_help_bar(frame: &mut Frame, area: Rect, is_connected: bool) {
        help_bar::draw(
            frame,
            area,
            if is_connected {
                vec![
                    "q: quit".to_string(),
                    "u: up".to_string(),
                    "d: down".to_string(),
                ]
            } else {
                vec!["q: quit".to_string()]
            },
        )
    }

    fn draw_movement_box(
        frame: &mut Frame,
        area: Rect,
        position: Option<PositionSpeed>,
        is_focused: bool,
    ) {
        let direction_text = match position {
            // fixme: it would be nice to substitute this with an arrow
            Some(ref p) => match p.get_direction() {
                Direction::Up => "(up)",
                Direction::Down => "(down)",
                Direction::Idle => "",
            },
            None => "",
        };

        let movement_text = BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .style(Style::new().blue())
            .lines(vec![Line::from(if let Some(ref v) = position {
                format!("{:.2}cm {}", v.position_to_cm(), direction_text)
            } else {
                "???".to_string()
            })])
            .build()
            .unwrap();

        let mut container = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .padding(Padding::new(4, 4, 2, 2))
            .title("Movement");
        if is_focused {
            container = container.border_style(Style::new().bold().light_red());
        }

        let inner_area = container.inner(area);

        frame.render_widget(container.clone(), area);
        frame.render_widget(movement_text, inner_area);
    }
}

impl App {
    pub fn handle_movement_event(&mut self, ev: UIEvent) {
        match ev {
            UIEvent::KeyPress(ev) => match ev.code {
                KeyCode::Char('u') => self.trigger_movement(MovingDirection::Up),
                KeyCode::Char('d') => self.trigger_movement(MovingDirection::Down),
                _ => (),
            },
            UIEvent::KeyRelease(ev) => match ev.code {
                KeyCode::Char('u') | KeyCode::Char('d') => {
                    self.trigger_movement(MovingDirection::Stop)
                }
                _ => (),
            },
            _ => (),
        }
    }

    pub fn draw_movement(&mut self, frame: &mut Frame, content_area: Rect, bar_area: Rect) {
        let state = self.get_current_state();
        MovementView::draw_movement_box(
            frame,
            content_area,
            state.position,
            state.current_view == Views::Movement,
        );

        if state.current_view == Views::Movement {
            MovementView::draw_help_bar(frame, bar_area, state.connected_device.is_some())
        }
    }
}
