use std::ops::Index;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

const HELP_LABEL_PADDING: u16 = 4;

pub fn draw(frame: &mut Frame, area: Rect, labels: Vec<String>) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let inner_area = block.inner(area);
    let labels: Vec<Text> = labels.into_iter().map(|t| Text::from(t)).collect();

    let layout = Layout::new(
        Direction::Horizontal,
        labels
            .iter()
            .map(|l| Constraint::Min(l.width() as u16 + HELP_LABEL_PADDING)),
    )
    .split(inner_area);

    frame.render_widget(block, area);

    for (idx, label) in labels.iter().enumerate() {
        frame.render_widget(Paragraph::new(label.to_owned()), *layout.index(idx));
    }
}
