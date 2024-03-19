use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::Text,
};

#[allow(dead_code)]
pub fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(outer[1])[1]
}

pub fn centered_rect_text(area: Rect, content: Text) -> Rect {
    let (content_height, content_width) = (content.height() + 2, content.width() + 2);
    let (x, y) = (
        usize::from(area.width / 2) - content_width / 2,
        usize::from(area.height / 2) - content_height / 2,
    );

    Rect {
        x: x as u16,
        y: y as u16,
        width: content_width as u16,
        height: content_height as u16,
    }
}

#[allow(dead_code)]
pub fn centered_rect_area(area: Rect, content: Rect) -> Rect {
    let (content_height, content_width) = (content.height + 2, content.width + 2);
    let (x, y) = (
        area.width / 2 - content_width / 2,
        area.height / 2 - content_height / 2,
    );

    Rect {
        x,
        y,
        width: content_width,
        height: content_height,
    }
}
