use std::{
    error::Error,
    sync::{atomic::AtomicBool, mpsc::Sender, Arc, Mutex},
    thread::JoinHandle,
};

use btleplug::api::Peripheral;
use crossterm::event::KeyCode;
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    text::Text,
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
    Frame, Terminal,
};

use crate::{
    bluetooth::manager::BleManager,
    tui::{centered_rect::centered_rect_text, notification_list::NotificationList},
};

use super::{
    actions::{AppAction, MovingDirection, SelectionMove},
    config::Config,
    jobs::{
        background_job::BackgroundJob,
        movement_job,
        notification_job::{Notification, NotificationManager},
    },
    state::{State, Status},
    ui_event_emitter::{EventEmitter, UIEvent},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Views {
    Connection,
    Movement,
    SavedPositions,
    DeviceList,
}

pub struct App {
    state: Arc<Mutex<State>>,

    notifications: Arc<Mutex<Vec<Notification>>>,

    long_running_actions_sx: Sender<AppAction>,
    _long_running_actions_handler: JoinHandle<()>,

    _notification_sx: Sender<Notification>,
    _notifications_handler: JoinHandle<()>,

    _movement_handler: JoinHandle<()>,

    exited: Arc<AtomicBool>,
}

impl App {
    pub fn new(ble_manager: BleManager, config: Config) -> Self {
        let state = Arc::new(Mutex::new(State::new(config.clone())));
        let notifications = Arc::new(Mutex::new(Vec::new()));
        let idasen = Arc::new(Mutex::new(None));
        let exited = Arc::new(AtomicBool::new(false));

        let notification_manager = NotificationManager::new(Arc::clone(&notifications));
        let _notification_sx = notification_manager.get_sender();
        let background_notification_sx = notification_manager.get_sender();
        let _notifications_handler = notification_manager.run();

        let _movement_handler =
            movement_job::MovementJob::new(Arc::clone(&idasen), Arc::clone(&state)).run();

        let background_job_executor = BackgroundJob::new(
            idasen,
            Arc::clone(&state),
            ble_manager,
            background_notification_sx,
            Arc::clone(&exited),
        );
        let action_sx = background_job_executor.get_sender();
        let _long_running_actions_handler = background_job_executor.run();

        action_sx.send(AppAction::Init(config.clone())).unwrap();

        Self {
            state,

            notifications,

            long_running_actions_sx: action_sx,
            _long_running_actions_handler,

            _notification_sx,
            _notifications_handler,

            _movement_handler,

            exited,
        }
    }

    pub async fn run<B: Backend>(
        mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Box<dyn Error>> {
        let events = EventEmitter::new(250);
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            let ev = events.next()?;
            let state = self.get_current_state();

            if let Status::Freezed(_) = state.status {
                continue;
            }

            if !self.is_global_event(&ev) {
                match state.current_view {
                    Views::Connection => (),
                    Views::Movement => self.handle_movement_event(ev),
                    Views::DeviceList => self.handle_device_list_event(ev),
                    Views::SavedPositions => self.handle_positions_event(ev),
                }
            }

            if self.exited.load(std::sync::atomic::Ordering::Relaxed) {
                // fix: this should not have unwrap
                return Ok(());
            }
        }
    }

    pub fn start_action(&self, action: AppAction) {
        self.long_running_actions_sx.send(action).unwrap()
    }

    pub fn move_device_selection(&self, action: SelectionMove) {
        match action {
            SelectionMove::Prev => self.state.lock().unwrap().devices_list.select_prev(),
            SelectionMove::Next => self.state.lock().unwrap().devices_list.select_next(),
        }
    }

    pub fn move_position_selection(&self, action: SelectionMove) {
        match action {
            SelectionMove::Prev => self.state.lock().unwrap().positions_list.select_prev(),
            SelectionMove::Next => self.state.lock().unwrap().positions_list.select_next(),
        }
    }

    pub fn trigger_movement(&self, mov: MovingDirection) {
        self.start_action(AppAction::StartMoving(mov))
    }

    pub fn connect_selected(&self) {
        if let Some(device) = self.state.lock().unwrap().devices_list.get_selected() {
            self.start_action(AppAction::ConnectDevice(device.peripheral.address()))
        }
    }

    pub fn move_view_focus_next(&self) {
        let mut state = self.state.lock().unwrap();
        state.current_view =
            Self::get_view_by_index((Self::get_index_by_view(state.current_view) + 1) % 4)
    }

    pub fn move_view_focus_prev(&self) {
        let mut state = self.state.lock().unwrap();
        let current_index = Self::get_index_by_view(state.current_view);
        state.current_view = Self::get_view_by_index(if current_index > 0 {
            current_index - 1
        } else {
            3
        })
    }

    pub fn get_current_state(&self) -> State {
        self.state.lock().unwrap().clone()
    }

    fn draw(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Max(3)].as_ref())
            .split(frame.size());

        let (view_area, bar_area) = (layout[0], layout[1]);

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(view_area);
        let (view_left_area, view_right_area) = (layout[0], layout[1]);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(view_left_area);
        let (view_left_top_area, view_left_bottom_area) = (layout[0], layout[1]);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(view_right_area);
        let (view_right_top_area, view_right_bottom_area) = (layout[0], layout[1]);

        self.draw_connections(frame, view_left_top_area, bar_area);
        self.draw_movement(frame, view_right_top_area, bar_area);
        self.draw_positions(frame, view_left_bottom_area, bar_area);
        self.draw_device_list(frame, view_right_bottom_area, bar_area);

        if let Status::Freezed(reason) | Status::Background(reason) =
            self.get_current_state().status
        {
            self.draw_freezing_message(frame, reason)
        }

        let notifications = if let Ok(list) = self.notifications.try_lock() {
            list.clone()
        } else {
            Vec::new()
        };

        NotificationList::draw(frame, notifications)
    }

    fn draw_freezing_message(&self, frame: &mut Frame, reason: String) {
        let box_area = centered_rect_text(frame.size(), Text::from(reason.clone()));
        frame.render_widget(Clear, box_area);
        frame.render_widget(
            Paragraph::new(reason).alignment(Alignment::Center).block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .padding(Padding::zero()),
            ),
            box_area,
        );
    }

    // fix: this is ugly since it has side effects
    fn is_global_event(&mut self, ev: &UIEvent) -> bool {
        if let UIEvent::KeyPress(ev) = ev {
            match ev.code {
                KeyCode::Char('q') => self.start_action(AppAction::TearDown),
                KeyCode::Tab => self.move_view_focus_next(),
                KeyCode::BackTab => self.move_view_focus_prev(),
                _ => (),
            }
        }
        false
    }

    fn get_view_by_index(idx: usize) -> Views {
        match idx {
            0 => Views::Connection,
            1 => Views::Movement,
            2 => Views::SavedPositions,
            3 => Views::DeviceList,
            _ => unreachable!(),
        }
    }

    fn get_index_by_view(view: Views) -> usize {
        match view {
            Views::Connection => 0,
            Views::Movement => 1,
            Views::SavedPositions => 2,
            Views::DeviceList => 3,
        }
    }
}
