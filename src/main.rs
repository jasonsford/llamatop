use std::io::stdout;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

mod client;
mod telemetry;
mod ui;

use client::llama_server::MultiLlamaManager;
use telemetry::create_collector;

#[derive(Parser, Debug)]
#[command(author, version, about = "Top monitor for llama.cpp and multi-GPU setups on Linux", long_about = None)]
struct Args {
    /// Refresh interval in milliseconds
    #[arg(short, long, default_value_t = 1000)]
    interval: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, crossterm::cursor::Hide)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let mut collector = create_collector();
    let mut llama_manager = MultiLlamaManager::new();

    let refresh_duration = Duration::from_millis(args.interval);
    let mut last_tick = std::time::Instant::now();

    let mut host_stats = collector.poll_host();
    let mut gpu_stats = collector.poll_gpus();
    let mut llama_stats = llama_manager.poll().await;

    loop {
        terminal.draw(|f| {
            ui::draw_dashboard(f, &host_stats, &gpu_stats, &llama_stats);
        })?;

        let timeout = refresh_duration.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= refresh_duration {
            host_stats = collector.poll_host();
            gpu_stats = collector.poll_gpus();
            llama_stats = llama_manager.poll().await;
            last_tick = std::time::Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        crossterm::cursor::Show,
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    Ok(())
}