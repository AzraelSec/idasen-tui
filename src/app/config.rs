use std::fs::File;

use dirs::home_dir;
use serde::{Deserialize, Serialize};

use super::state::SavedPosition;

const CONFIG_PATH: &str = ".idasen-tui.json";

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    // note: this is needed because of the way Serde is used in btleplug
    pub predefined_mac: Option<String>,
    pub saved_positions: Vec<SavedPosition>,
}

impl Config {
    fn default() -> Self {
        Self {
            predefined_mac: None,
            saved_positions: Vec::new(),
        }
    }

    pub fn load_from(path: String) -> Self {
        match File::open(path) {
            Ok(f) => serde_json::from_reader(f).map_or_else(|_| Self::default(), |c| c),
            Err(_) => Self::default(),
        }
    }

    pub fn load() -> Self {
        if let Some(home) = home_dir() {
            Self::load_from(home.join(CONFIG_PATH).to_string_lossy().into_owned())
        } else {
            Self::default()
        }
    }
}
