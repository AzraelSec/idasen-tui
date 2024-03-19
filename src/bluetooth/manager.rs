use std::time::Duration;

use btleplug::api::{BDAddr, Central, Manager as _, ParseBDAddrError, Peripheral, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use tokio;

use super::ble_device::BleDevice;

#[derive(Clone)]
pub struct BleManager {
    low_level_manager: Manager,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Cannot find the device.")]
    CannotFindDevice,

    #[error("errored to parse mac address.")]
    MacAddrParseFailed(#[from] ParseBDAddrError),

    #[error("bluetooth error {0}")]
    BtlePlugError(#[from] btleplug::Error),
}

impl BleManager {
    pub async fn new() -> Result<Self, Error> {
        let low_level_manager = Manager::new().await?;
        Ok(Self { low_level_manager })
    }

    pub async fn find(&self, addr: BDAddr) -> Result<BleDevice, Error> {
        let device = self
            .scan()
            .await?
            .into_iter()
            .find(|d| d.peripheral.address() == addr)
            .ok_or_else(|| Error::CannotFindDevice)?;
        Ok(device)
    }

    pub async fn scan(&self) -> Result<Vec<BleDevice>, Error> {
        let adapters = self.low_level_manager.adapters().await?;
        let mut jobs = Vec::new();

        for adapter in adapters {
            jobs.push(tokio::spawn(async move {
                BleManager::collect_peripherals(adapter).await
            }));
        }

        let mut peripherals = Vec::new();
        for job in jobs {
            if let Ok(Ok(mut found_peripherals)) = job.await {
                peripherals.append(&mut found_peripherals);
            }
        }

        Ok(peripherals)
    }

    async fn collect_peripherals(adapter: Adapter) -> Result<Vec<BleDevice>, Error> {
        adapter.start_scan(ScanFilter::default()).await?;
        tokio::time::sleep(Duration::from_secs(5)).await;

        let mut items = Vec::new();
        for peripheral in adapter.peripherals().await? {
            if let Ok(device) = BleDevice::new(peripheral).await {
                items.push(device);
            }
        }

        adapter.stop_scan().await?;
        Ok(items)
    }
}
