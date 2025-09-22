use clap::Parser;
use color_eyre::Result;
use colored::Colorize;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    prelude::*,
    widgets::Paragraph,
};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "DIR")]
    path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct App {
    /// Path to the torrent file
    pub path: PathBuf,

    /// Name of the application
    pub app_name: String,

    /// Whether the app should exit
    pub should_quit: bool,

    /// Current selected index for navigation in the UI
    pub selected_index: usize,
}

impl App {
    pub fn new(path: PathBuf, app_name: String) -> Self {
        Self {
            path,
            app_name,
            should_quit: false,
            selected_index: 0,
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn next(&mut self) {
        self.selected_index = self.selected_index.saturating_add(1);
    }

    pub fn previous(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    pub fn handle_key_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.quit(),
            KeyCode::Char('p') => self.previous(),
            KeyCode::Char('n') => self.next(),
            KeyCode::Esc => self.quit(),
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = args.path.unwrap_or_else(|| {
        eprintln!("Path not provided, using current directory");
        PathBuf::from(".")
    });

    color_eyre::install()?;
    let terminal = ratatui::init();
    let app = App::new(path, "BitTorrent Clone".to_string());
    let result = run(terminal, app);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|frame| render(frame, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.handle_key_input(key.code);
            }

            if app.should_quit {
                break;
            }
        }
    }
    Ok(())
}

fn render(frame: &mut Frame, app: &App) {
    let text = Paragraph::new(format!(
        "BitTorrent Clone - Press 'q' or 'Esc' to quit\nPath: {}\nSelected Index: {}",
        app.path.display(),
        app.selected_index
    ));
    frame.render_widget(text, frame.area());
}
