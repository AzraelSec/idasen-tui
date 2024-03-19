mod app;
mod bluetooth;
mod idasen;
mod tui;

use std::io;

use app::{app::App, config::Config};
use bluetooth::manager::BleManager;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

#[tokio::main]
async fn main() {
    let ble_manager = BleManager::new().await.unwrap();

    let mut stdout = &io::stdout();
    enable_raw_mode().unwrap();
    execute!(stdout, EnterAlternateScreen).unwrap();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let app = App::new(ble_manager, Config::load());
    app.run(&mut terminal).await.unwrap();

    disable_raw_mode().unwrap();
    execute!(stdout, LeaveAlternateScreen).unwrap();
    terminal.show_cursor().unwrap();
}

