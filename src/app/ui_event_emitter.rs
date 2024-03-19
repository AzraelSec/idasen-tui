use std::{
    error::Error,
    sync::mpsc,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crossterm::event::{self, Event as CrossTermEvent, KeyEvent, KeyEventKind, MouseEvent};

pub enum UIEvent {
    Tick,
    KeyPress(KeyEvent),
    KeyRelease(KeyEvent),
    Click(MouseEvent),
    #[allow(dead_code)]
    Resize(u16, u16),
}

pub struct EventEmitter {
    _sender: mpsc::Sender<UIEvent>,
    receiver: mpsc::Receiver<UIEvent>,
    _handler: JoinHandle<()>,
}

impl EventEmitter {
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sx, rx) = mpsc::channel::<UIEvent>();

        let handler = {
            let sx = sx.clone();

            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(tick_rate);

                    if let Ok(true) = event::poll(timeout) {
                        let _ = match event::read() {
                            Ok(ev) => match ev {
                                CrossTermEvent::Key(e) => match e.kind {
                                    KeyEventKind::Press => sx.send(UIEvent::KeyPress(e)),
                                    KeyEventKind::Release => sx.send(UIEvent::KeyRelease(e)),
                                    _ => Ok(()),
                                },
                                CrossTermEvent::Mouse(e) => sx.send(UIEvent::Click(e)),
                                _ => Ok(()),
                            },
                            _ => Ok(()),
                        };
                    }

                    if last_tick.elapsed() >= tick_rate {
                        let _ = sx.send(UIEvent::Tick);
                        last_tick = Instant::now()
                    }
                }
            })
        };

        Self {
            _sender: sx,
            receiver: rx,
            _handler: handler,
        }
    }

    pub fn next(&self) -> Result<UIEvent, Box<dyn Error>> {
        Ok(self.receiver.recv()?)
    }
}
