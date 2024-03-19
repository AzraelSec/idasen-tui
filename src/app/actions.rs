use btleplug::api::BDAddr;

use super::config::Config;

pub enum AppAction {
    Init(Config),
    TearDown,

    UpdateDevicesList,
    ConnectDevice(BDAddr),

    StartMoving(MovingDirection),
}

pub enum MovingDirection {
    Up,
    Down,
    Stop,
    ToHeight(u16),
}

pub enum SelectionMove {
    Prev,
    Next,
}
