use crossterm::{
    event, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dotenvy::dotenv;
use log::info;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{env, io, time::Duration};
use tokio::{select, sync::mpsc, time::sleep};
mod core;
mod logs;
mod ui;

#[tokio::main]
async fn main() -> io::Result<()> {
    logs::log::init_logger();
    dotenv().ok();
    let ip = env::var("IP").unwrap();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, mut rx) = mpsc::channel(100);

    tokio::spawn(async move { core::port::scan_ports(tx, &ip).await });

    let mut results = vec![];

    loop {
        select! {
            Some(result) = rx.recv() => {
                results.push(result.clone());
                info!("{}", result);
            }
            _ = sleep(Duration::from_millis(100)) => {}
        }

        ui::ui::draw_ui(&mut terminal, &results)?;

        if event::poll(Duration::from_millis(50))? {
            if let event::Event::Key(key) = event::read()? {
                if key.code == event::KeyCode::Char('q') {
                    info!("User pressed 'q', exiting...");
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
