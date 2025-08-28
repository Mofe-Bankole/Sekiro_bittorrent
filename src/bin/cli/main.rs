use clap::Parser;
use color_eyre::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode},
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
            _ => {}
        }
    }

    // pub fn handle_mouse_evene
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = args.path.expect("Path not provided");
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    loop {
        terminal.draw(render)?;
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    let text = Paragraph::new("hello world");
    frame.render_widget(text, frame.area());
}
