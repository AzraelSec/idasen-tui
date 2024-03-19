use btleplug::api::BDAddr;
use serde::{Deserialize, Serialize};

use crate::{
    bluetooth::ble_device::BleDevice,
    idasen::idasen::PositionSpeed,
    tui::list::{ListableItem, StatefulList},
};

use super::{app::Views, config::Config};

#[derive(Clone)]
pub enum Status {
    Background(String),
    Freezed(String),
    Running,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SavedPosition {
    pub name: String,
    pub height: u16,
}

impl ListableItem for SavedPosition {
    fn render_row(&self) -> String {
        format!(
            "{} ({:.2}cm)",
            self.name.to_string(),
            self.height as f32 * 0.01
        )
    }

    fn is_highlighted(&self) -> bool {
        false
    }
}

#[derive(Clone)]
pub struct State {
    pub positions_list: StatefulList<SavedPosition>,
    pub devices_list: StatefulList<BleDevice>,
    pub current_view: Views,
    pub status: Status,
    pub connected_device: Option<BDAddr>,
    pub position: Option<PositionSpeed>,
}

impl State {
    pub fn new(config: Config) -> Self {
        Self {
            positions_list: StatefulList::with_items(config.saved_positions),
            devices_list: StatefulList::new(),
            current_view: Views::Connection,
            status: Status::Running,
            connected_device: None,
            position: None,
        }
    }
}
