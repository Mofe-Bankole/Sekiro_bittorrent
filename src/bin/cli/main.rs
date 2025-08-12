use std::io;

use clap::Parser;
use ratatui::{DefaultTerminal, Frame, Terminal, prelude::Backend};

#[derive(Parser, Debug)]
struct App {
    path: Vec<String>,
}

impl App {
    fn new() -> Self {
        App { path: Vec::new() }
    }

    fn run_app<B: Backend>(&mut self, terminal: &mut Terminal, mut app: App) -> io::Result {
        loop {
            tokio::spawn(async move {
                terminal
                    .draw(|f| {
                        let size = f.size();
                    })
                    .await;
            });
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    let app = App::new();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
}
