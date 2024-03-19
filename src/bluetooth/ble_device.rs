use std::error::Error;

use btleplug::{
    api::{Peripheral as _, PeripheralProperties},
    platform::Peripheral,
};

use crate::tui::list::ListableItem;

#[derive(Clone)]
pub struct BleDevice {
    pub peripheral: Peripheral,
    pub properties: PeripheralProperties,
}

impl BleDevice {
    pub async fn new(peripheral: Peripheral) -> Result<Self, Box<dyn Error>> {
        let properties = peripheral.properties().await?.unwrap_or_default();
        Ok(Self {
            peripheral,
            properties,
        })
    }
}

impl ListableItem for BleDevice {
    fn render_row(&self) -> String {
        format!(
            "{} ({})",
            self.properties
                .local_name
                .clone()
                .unwrap_or("unknown".to_owned()),
            self.properties.address
        )
    }

    fn is_highlighted(&self) -> bool {
        false //self.is_connected
    }
}
