use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
pub enum NotificationType {
    Info,
    #[allow(dead_code)]
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct Notification {
    pub kind: NotificationType,
    pub message: String,
}

impl Notification {
    pub fn new(kind: NotificationType, message: String) -> Self {
        Self { kind, message }
    }
}

#[derive(Clone, Debug)]
pub struct NotificationWrapper {
    payload: Notification,
    created_at: Instant,
}

impl NotificationWrapper {
    fn new(n: Notification) -> Self {
        Self {
            created_at: Instant::now(),
            payload: n,
        }
    }

    fn is_expired(&self, now: Instant) -> bool {
        now.duration_since(self.created_at) > NOTIFICATION_TTL
    }
}

pub struct NotificationManager {
    visible_notifications: Arc<Mutex<Vec<Notification>>>,
    internal_queue: Vec<NotificationWrapper>,

    requests_rx: Receiver<Notification>,
    _requests_sx: Sender<Notification>,
}

const NOTIFICATION_VALIDATION_DELAY: Duration = Duration::new(2, 0);
const NOTIFICATION_TTL: Duration = Duration::new(5, 0);

impl NotificationManager {
    pub fn new(visible_notifications: Arc<Mutex<Vec<Notification>>>) -> Self {
        let (sx, rx) = mpsc::channel::<Notification>();

        Self {
            visible_notifications,
            internal_queue: Vec::new(),

            requests_rx: rx,
            _requests_sx: sx,
        }
    }

    pub fn get_sender(&self) -> Sender<Notification> {
        self._requests_sx.clone()
    }

    pub fn run(mut self) -> JoinHandle<()> {
        thread::spawn(move || loop {
            let now = Instant::now();

            while let Ok(new_notification) = self.requests_rx.try_recv() {
                self.internal_queue
                    .push(NotificationWrapper::new(new_notification));
            }

            self.internal_queue = self
                .internal_queue
                .into_iter()
                .filter(|elem| !elem.is_expired(now))
                .collect();

            if let Ok(mut visible_notifications) = self.visible_notifications.try_lock() {
                *visible_notifications = self
                    .internal_queue
                    .clone()
                    .into_iter()
                    .map(|wrapper| wrapper.payload)
                    .collect();
            }

            thread::sleep(NOTIFICATION_VALIDATION_DELAY);
        })
    }
}
