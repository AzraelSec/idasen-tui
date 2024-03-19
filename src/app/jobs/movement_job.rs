use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use tokio_stream::StreamExt;

use crate::{app::state::State, idasen::idasen::Idasen};

pub struct MovementJob {
    idasen: Arc<Mutex<Option<Idasen>>>,
    state: Arc<Mutex<State>>,
}

const MOVEMENT_UPDATE_DELAY: Duration = Duration::new(2, 0);

impl MovementJob {
    pub fn new(idasen: Arc<Mutex<Option<Idasen>>>, state: Arc<Mutex<State>>) -> Self {
        Self { idasen, state }
    }

    pub fn run(self) -> JoinHandle<()> {
        thread::spawn(|| self.main_logic())
    }

    #[tokio::main]
    async fn main_logic(self) {
        loop {
            let mut stream = None;
            if let Some(ref idasen) = *self.idasen.try_lock().unwrap() {
                if let Ok(ps) = idasen.position_and_speed().await {
                    self.state.lock().unwrap().position = Some(ps)
                }
                if let Ok(s) = idasen.position_and_speed_stream().await {
                    stream = Some(s);
                }
            } else {
                self.state.lock().unwrap().position = None
            }

            if let Some(mut stream) = stream {
                while let Some(ps) = stream.next().await {
                    self.state.lock().unwrap().position = Some(ps)
                }
            }

            thread::sleep(MOVEMENT_UPDATE_DELAY);
        }
    }
}
