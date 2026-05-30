use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

pub struct App {
    pub running: bool,
}

impl App {
    pub fn new() -> Self {
        App { running: true }
    }

    pub fn handle_event(&mut self, event: &Event) {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.running = false,
                _ => {}
            }
        }
    }
}

struct TerminalGuard;

impl TerminalGuard {
    fn setup() -> Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        Ok(TerminalGuard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    }
}

pub fn run() -> Result<()> {
    let _guard = TerminalGuard::setup()?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();

    while app.running {
        terminal.draw(|_frame| {
            // blank frame — content added in T9/T10
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            app.handle_event(&event::read()?);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    #[test]
    fn app_starts_running() {
        let app = App::new();
        assert!(app.running);
    }

    #[test]
    fn q_key_stops_running() {
        let mut app = App::new();
        app.handle_event(&key(KeyCode::Char('q')));
        assert!(!app.running);
    }

    #[test]
    fn esc_key_stops_running() {
        let mut app = App::new();
        app.handle_event(&key(KeyCode::Esc));
        assert!(!app.running);
    }

    #[test]
    fn other_key_keeps_running() {
        let mut app = App::new();
        app.handle_event(&key(KeyCode::Char('a')));
        assert!(app.running);
    }
}
