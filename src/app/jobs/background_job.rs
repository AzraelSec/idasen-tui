use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use btleplug::api::BDAddr;

use crate::{
    app::{
        actions::{AppAction, MovingDirection},
        state::{State, Status},
    },
    bluetooth::manager::BleManager,
    idasen::idasen::Idasen,
};

use super::notification_job::{Notification, NotificationType};

pub struct BackgroundJob {
    idasen: Arc<Mutex<Option<Idasen>>>,
    state: Arc<Mutex<State>>,
    ble_manager: BleManager,
    notification_sx: Sender<Notification>,
    exit: Arc<AtomicBool>,

    requests_rx: Receiver<AppAction>,
    requests_sx: Sender<AppAction>,
}

impl BackgroundJob {
    pub fn new(
        idasen: Arc<Mutex<Option<Idasen>>>,
        state: Arc<Mutex<State>>,
        ble_manager: BleManager,
        notification_sx: Sender<Notification>,
        exit: Arc<AtomicBool>,
    ) -> Self {
        let (requests_sx, requests_rx) = mpsc::channel::<AppAction>();
        Self {
            idasen,
            state,
            ble_manager,
            notification_sx,
            exit,

            requests_rx,
            requests_sx,
        }
    }

    pub fn get_sender(&self) -> Sender<AppAction> {
        self.requests_sx.clone()
    }

    pub fn run(self) -> thread::JoinHandle<()> {
        thread::spawn(|| self.main_logic())
    }

    #[tokio::main]
    async fn main_logic(mut self) {
        loop {
            match self.requests_rx.recv() {
                Ok(AppAction::Init(config)) => {
                    if let Some(addr) = config.predefined_mac {
                        if let Ok(addr) = BDAddr::from_str(&addr) {
                            self.connect(addr).await
                        } else {
                            self.show_error("invalid mac address in config")
                        }
                    };
                    self.update_devices().await;
                }
                Ok(AppAction::UpdateDevicesList) => self.update_devices().await,
                Ok(AppAction::ConnectDevice(addr)) => self.connect(addr).await,
                Ok(AppAction::StartMoving(direction)) => self.move_desk_to(direction).await,
                Ok(AppAction::TearDown) => {
                    self.tear_down().await;
                    return;
                }
                _ => (),
            }
        }
    }

    async fn update_devices(&mut self) {
        {
            self.state.lock().unwrap().status =
                Status::Background("loading devices list...".to_string())
        }

        if let Ok(new_devices) = self.ble_manager.scan().await {
            self.state.lock().unwrap().devices_list.set(new_devices);
            self.show_notification(NotificationType::Info, "devices scan completed")
        } else {
            self.show_error("error in running bluetooth scan")
        }

        {
            self.state.lock().unwrap().status = Status::Running;
        }
    }

    async fn connect(&mut self, addr: BDAddr) {
        {
            self.state.lock().unwrap().status =
                Status::Background("connecting to device...".to_string());
        }

        if let Ok(device) = self.ble_manager.find(addr).await {
            let _idasen_val = match Idasen::new(device.peripheral).await {
                Ok(x) => Some(x),
                Err(_) => None,
            };
            let addr = _idasen_val
                .as_ref()
                .map_or(None, |device| Some(device.mac_addr));

            *self.idasen.lock().unwrap() = _idasen_val;
            self.state.lock().unwrap().connected_device = addr;

            self.show_notification(NotificationType::Info, "device succesfully connected")
        } else {
            self.show_error("impossible to find the given device");
        };

        self.state.lock().unwrap().status = Status::Running;
    }

    // todo: handle errors with notifications
    async fn move_desk_to(&mut self, direction: MovingDirection) {
        if let Some(ref _idasen) = *self.idasen.lock().unwrap() {
            match direction {
                MovingDirection::ToHeight(height) => _idasen.move_to(height).await.unwrap(),
                MovingDirection::Up => {
                    {
                        self.state.lock().unwrap().status =
                            Status::Freezed("desk is moving...".to_string());
                    }
                    _idasen.up().await.unwrap();
                    self.state.lock().unwrap().status = Status::Running;
                }
                MovingDirection::Down => _idasen.down().await.unwrap(),
                // fix: this never takes place because the up/down ops are sync
                // note: it would be nice if these operations occurred without awaiting
                // them
                MovingDirection::Stop => _idasen.stop().await.unwrap(),
            }
        }
    }

    async fn tear_down(&mut self) {
        if let Some(ref _idasen) = *self.idasen.lock().unwrap() {
            {
                self.state.lock().unwrap().status = Status::Freezed("shutting down...".to_string())
            }
            if let Err(_) = _idasen.disconnect().await {
                self.show_error("impossible to disconnect from device")
            } else {
                self.show_notification(NotificationType::Info, "device disconnected")
            };
            self.state.lock().unwrap().status = Status::Running;
        }
        self.exit.swap(true, Ordering::Relaxed);
    }

    fn show_error(&self, msg: &str) {
        self.show_notification(NotificationType::Error, msg)
    }

    fn show_notification(&self, kind: NotificationType, msg: &str) {
        let _ = self
            .notification_sx
            .send(Notification::new(kind, msg.to_string()));
    }
}
