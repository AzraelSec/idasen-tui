use std::cmp;

use ratatui::{
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::jobs::notification_job::{Notification, NotificationType};

const MAX_ITEM_HEIGHT: u16 = 5;
const MAX_ITEM_WIDTH: u16 = 45;

const NOTIFICATION_MARGIN: u16 = 1;
const NOTIFICATION_PADDING: u16 = 4; // note: this could probably be calculated

pub struct NotificationList;

impl NotificationList {
    pub fn draw(window_frame: &mut Frame, to_show: Vec<Notification>) {
        let items: Vec<(Paragraph, Rect)> = to_show
            .into_iter()
            .enumerate()
            .map(|(idx, notification)| {
                let text = Text::from(notification.message);
                let paragraph = Paragraph::new(text.clone())
                    .alignment(Alignment::Center)
                    .wrap(Wrap::default())
                    .block(
                        Block::default()
                            .title(match notification.kind {
                                NotificationType::Info => "info",
                                NotificationType::Error => "error!",
                                NotificationType::Warning => "warning!",
                            })
                            .border_type(BorderType::Rounded)
                            .borders(Borders::ALL)
                            .border_style(match notification.kind {
                                NotificationType::Info => Style::default().blue(),
                                NotificationType::Warning => Style::default().yellow(),
                                NotificationType::Error => Style::default().red(),
                            }),
                    );

                let item_width =
                    cmp::min(MAX_ITEM_WIDTH, (text.width() as u16) + NOTIFICATION_PADDING);
                let item_height = cmp::min(MAX_ITEM_HEIGHT, (text.height() as u16) + NOTIFICATION_PADDING);

                let frame = Rect::new(
                    window_frame.size().width - item_width - NOTIFICATION_MARGIN,
                    window_frame.size().height
                        - ((idx as u16) + NOTIFICATION_MARGIN) * (item_height),
                    item_width,
                    item_height,
                );
                (paragraph, frame)
            })
            .collect();

        for (paragraph, frame) in items.into_iter() {
            window_frame.render_widget(Clear, frame);
            window_frame.render_widget(paragraph, frame);
        }
    }
}
