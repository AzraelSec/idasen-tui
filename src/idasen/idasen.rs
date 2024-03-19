/*
 * Original code adapted from Idasen
 * License: MIT
 * Author: aklajnert
 * Repository: https://github.com/aklajnert/idasen
*/
pub use btleplug::api::Peripheral as Device;
use btleplug::api::{BDAddr, Characteristic, ParseBDAddrError, WriteType};
use btleplug::platform::Peripheral;
use std::{
    cmp::{max, Ordering},
    time::Instant,
};

use tokio_stream::{Stream, StreamExt};
use uuid::Uuid;

const CONTROL_UUID: Uuid = Uuid::from_bytes([
    0x99, 0xfa, 0x00, 0x02, 0x33, 0x8a, 0x10, 0x24, 0x8a, 0x49, 0x00, 0x9c, 0x02, 0x15, 0xf7, 0x8a,
]);
const POSITION_UUID: Uuid = Uuid::from_bytes([
    0x99, 0xfa, 0x00, 0x21, 0x33, 0x8a, 0x10, 0x24, 0x8a, 0x49, 0x00, 0x9c, 0x02, 0x15, 0xf7, 0x8a,
]);

const UP: [u8; 2] = [0x47, 0x00];
const DOWN: [u8; 2] = [0x46, 0x00];
const STOP: [u8; 2] = [0xFF, 0x00];

pub const MIN_HEIGHT: u16 = 6200;
pub const MAX_HEIGHT: u16 = 12700;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PositionSpeed {
    // tenth mm
    pub position: u16,
    // unknown
    pub speed: i16,
}

#[derive(Debug)]
pub enum Direction {
    Idle,
    Up,
    Down,
}

impl PositionSpeed {
    fn from_bytes(bytes: &[u8]) -> Self {
        let position = u16::from_le_bytes([bytes[0], bytes[1]]) + MIN_HEIGHT;
        let speed = i16::from_le_bytes([bytes[2], bytes[3]]);
        PositionSpeed { position, speed }
    }

    pub fn position_to_cm(&self) -> f32 {
        self.position as f32 * 0.01
    }

    pub fn get_direction(&self) -> Direction {
        match self.speed {
            0 => Direction::Idle,
            x if x > 0 => Direction::Up,
            x if x < 0 => Direction::Down,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Bluetooth characteristics not found: '{}'.", _0)]
    CharacteristicsNotFound(String),

    #[error("Desired position has to be between MIN_HEIGHT and MAX_HEIGHT.")]
    PositionNotInRange,

    #[error("Cannot subscribe to read position.")]
    CannotSubscribePosition,

    #[error("errored to parse mac address.")]
    MacAddrParseFailed(#[from] ParseBDAddrError),

    #[error("bluetooth error {0}")]
    BtlePlugError(#[from] btleplug::Error),
}

pub struct Idasen {
    pub mac_addr: BDAddr,
    device: Peripheral,
    control_characteristic: Characteristic,
    position_characteristic: Characteristic,
}

impl Idasen {
    /// Instantiate the struct. Requires `Device` instance.
    pub async fn new(desk: Peripheral) -> Result<Self, Error> {
        let mac_addr = desk.address();

        if !desk.is_connected().await? {
            desk.connect().await?;
        }
        desk.discover_services().await?;

        let control_characteristic = desk
            .characteristics()
            .iter()
            .find(|c| c.uuid == CONTROL_UUID)
            .ok_or_else(|| Error::CharacteristicsNotFound("Control".to_string()))?
            .clone();

        let position_characteristic = desk
            .characteristics()
            .iter()
            .find(|c| c.uuid == POSITION_UUID)
            .ok_or_else(|| Error::CharacteristicsNotFound("Position".to_string()))?
            .clone();

        if desk.subscribe(&position_characteristic).await.is_err() {
            return Err(Error::CannotSubscribePosition);
        };

        Ok(Self {
            device: desk,
            mac_addr,
            control_characteristic,
            position_characteristic,
        })
    }

    pub async fn disconnect(&self) -> Result<(), Error> {
        if self.device.is_connected().await? {
            self.device.disconnect().await?;
        }
        Ok(())
    }

    /// Move desk up.
    pub async fn up(&self) -> btleplug::Result<()> {
        self.device
            .write(
                &self.control_characteristic,
                &UP,
                WriteType::WithoutResponse,
            )
            .await
    }

    /// Lower the desk's position.
    pub async fn down(&self) -> btleplug::Result<()> {
        self.device
            .write(
                &self.control_characteristic,
                &DOWN,
                WriteType::WithoutResponse,
            )
            .await
    }

    /// Stop desk from moving.
    pub async fn stop(&self) -> btleplug::Result<()> {
        self.device
            .write(
                &self.control_characteristic,
                &STOP,
                WriteType::WithoutResponse,
            )
            .await
    }

    /// Move desk to a desired position. The precision is decent, usually less than 1mm off.
    pub async fn move_to(&self, target_position: u16) -> Result<(), Error> {
        self.move_to_target(target_position).await
    }

    async fn move_to_target(&self, target_position: u16) -> Result<(), Error> {
        if !(MIN_HEIGHT..=MAX_HEIGHT).contains(&target_position) {
            return Err(Error::PositionNotInRange);
        }

        let mut position_reached = false;
        let mut last_position = self.position().await? as i16;
        let mut last_position_read_at = Instant::now();
        let target_position = target_position as i16;
        while !position_reached {
            let current_position = self.position().await? as i16;
            let going_up = match target_position.cmp(&current_position) {
                Ordering::Greater => true,
                Ordering::Less => false,
                Ordering::Equal => return Ok(()),
            };
            let remaining_distance = (target_position - current_position).abs();
            let elapsed_millis = last_position_read_at.elapsed().as_millis();
            let moved_height = (last_position - current_position).abs();

            // Tenth of millimetres per second
            let speed = ((moved_height as f64 / elapsed_millis as f64) * 1000f64) as i16;

            if remaining_distance <= 10 {
                // Millimetre or less is good enough.
                position_reached = true;
                self.stop().await?;
            } else if going_up {
                self.up().await?;
            } else if !going_up {
                self.down().await?;
            }

            // If we're either:
            // * less than 5 millimetres, or:
            // * less than half a second from target
            // then we need to stop every iteration so that we don't overshoot
            if remaining_distance < max(speed / 2, 50) {
                self.stop().await?;
            }

            // Read last_position again to avoid weird speed readings when switching direction
            last_position = self.position().await? as i16;
            last_position_read_at = Instant::now();
        }

        Ok(())
    }

    /// Return the desk height in tenth millimeters (1m = 10000)
    pub async fn position(&self) -> Result<u16, Error> {
        Ok(self.position_and_speed().await?.position)
    }

    /// Return the denk height in tenth millimeters and speed in unknown dimension
    pub async fn position_and_speed(&self) -> Result<PositionSpeed, Error> {
        let value = self.device.read(&self.position_characteristic).await?;
        Ok(PositionSpeed::from_bytes(&value))
    }

    /// Listen to position and speed changes
    pub async fn position_and_speed_stream(
        &self,
    ) -> Result<impl Stream<Item = PositionSpeed>, Error> {
        Ok(self
            .device
            .notifications()
            .await?
            .filter_map(|notification| {
                if notification.uuid == POSITION_UUID {
                    Some(PositionSpeed::from_bytes(&notification.value))
                } else {
                    None
                }
            }))
    }
}
