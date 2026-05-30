use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::io;

use crate::storage::Prompt;

pub struct App {
    pub running: bool,
    pub prompts: Vec<Prompt>,
    pub query: String,
    pub filtered: Vec<usize>,
    pub selected: usize,
}

impl App {
    pub fn new(prompts: Vec<Prompt>) -> Self {
        let filtered = (0..prompts.len()).collect();
        App {
            running: true,
            prompts,
            query: String::new(),
            filtered,
            selected: 0,
        }
    }

    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.running = false,
                KeyCode::Char(c) => {
                    self.query.push(c);
                    self.refilter();
                }
                KeyCode::Backspace => {
                    self.query.pop();
                    self.refilter();
                }
                KeyCode::Down => {
                    if !self.filtered.is_empty() && self.selected + 1 < self.filtered.len() {
                        self.selected += 1;
                    }
                }
                KeyCode::Up => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                }
                _ => {}
            },
            Event::Mouse(mouse_event) => match mouse_event.kind {
                MouseEventKind::ScrollDown => {
                    if !self.filtered.is_empty() && self.selected + 1 < self.filtered.len() {
                        self.selected += 1;
                    }
                }
                MouseEventKind::ScrollUp => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn refilter(&mut self) {
        if self.query.is_empty() {
            self.filtered = (0..self.prompts.len()).collect();
        } else {
            let matcher = SkimMatcherV2::default();
            let mut scored: Vec<(i64, usize)> = self
                .prompts
                .iter()
                .enumerate()
                .filter_map(|(i, p)| {
                    let haystack = format!(
                        "{} {} {} {}",
                        p.id,
                        p.title,
                        p.description,
                        p.tags.join(" ")
                    );
                    matcher.fuzzy_match(&haystack, &self.query).map(|s| (s, i))
                })
                .collect();
            scored.sort_by(|a, b| b.0.cmp(&a.0));
            self.filtered = scored.into_iter().map(|(_, i)| i).collect();
        }
        if self.filtered.is_empty() {
            self.selected = 0;
        } else {
            self.selected = self.selected.min(self.filtered.len() - 1);
        }
    }
}

fn render(frame: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(frame.area());

    let search_text = format!("{}█", app.query);
    let search_widget = Paragraph::new(search_text)
        .block(Block::default().borders(Borders::ALL).title("Search"));
    frame.render_widget(search_widget, chunks[0]);

    let items: Vec<ListItem> = app
        .filtered
        .iter()
        .map(|&idx| {
            let p = &app.prompts[idx];
            let content = format!("[{}] {} — {}", p.category, p.id, p.title);
            ListItem::new(Line::from(Span::raw(content)))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Prompts"))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default();
    if !app.filtered.is_empty() {
        list_state.select(Some(app.selected));
    }
    frame.render_stateful_widget(list, chunks[1], &mut list_state);
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

    let prompts = crate::storage::load_prompts().unwrap_or_default();
    let mut app = App::new(prompts);

    while app.running {
        terminal.draw(|frame| {
            render(frame, &app);
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
    use crossterm::event::{KeyEvent, KeyModifiers, MouseEvent};

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn mouse_scroll(kind: MouseEventKind) -> Event {
        Event::Mouse(MouseEvent {
            kind,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        })
    }

    fn make_prompts() -> Vec<Prompt> {
        vec![
            Prompt {
                id: "perf-review".to_string(),
                title: "Performance Review".to_string(),
                category: "work".to_string(),
                description: "Review performance metrics".to_string(),
                prompt: "Review performance...".to_string(),
                tags: vec!["performance".to_string()],
                created_at: "2026-05-30T00:00:00Z".to_string(),
                updated_at: "2026-05-30T00:00:00Z".to_string(),
            },
            Prompt {
                id: "code-review".to_string(),
                title: "Code Review".to_string(),
                category: "dev".to_string(),
                description: "Review code quality".to_string(),
                prompt: "Review the code...".to_string(),
                tags: vec!["code".to_string()],
                created_at: "2026-05-30T00:00:00Z".to_string(),
                updated_at: "2026-05-30T00:00:00Z".to_string(),
            },
            Prompt {
                id: "write-tests".to_string(),
                title: "Write Tests".to_string(),
                category: "dev".to_string(),
                description: "Write unit tests".to_string(),
                prompt: "Write tests for...".to_string(),
                tags: vec!["testing".to_string()],
                created_at: "2026-05-30T00:00:00Z".to_string(),
                updated_at: "2026-05-30T00:00:00Z".to_string(),
            },
        ]
    }

    // ── existing behaviour ──────────────────────────────────────────────────

    #[test]
    fn app_starts_running() {
        let app = App::new(vec![]);
        assert!(app.running);
    }

    #[test]
    fn q_key_stops_running() {
        let mut app = App::new(vec![]);
        app.handle_event(&key(KeyCode::Char('q')));
        assert!(!app.running);
    }

    #[test]
    fn esc_key_stops_running() {
        let mut app = App::new(vec![]);
        app.handle_event(&key(KeyCode::Esc));
        assert!(!app.running);
    }

    #[test]
    fn non_quit_key_keeps_running() {
        let mut app = App::new(vec![]);
        app.handle_event(&key(KeyCode::Char('a')));
        assert!(app.running);
    }

    // ── search query ────────────────────────────────────────────────────────

    #[test]
    fn new_app_has_empty_query() {
        let app = App::new(make_prompts());
        assert_eq!(app.query, "");
    }

    #[test]
    fn new_app_shows_all_prompts() {
        let app = App::new(make_prompts());
        assert_eq!(app.filtered.len(), 3);
    }

    #[test]
    fn new_app_selection_starts_at_zero() {
        let app = App::new(make_prompts());
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn typing_char_appends_to_query() {
        let mut app = App::new(make_prompts());
        app.handle_event(&key(KeyCode::Char('p')));
        assert_eq!(app.query, "p");
    }

    #[test]
    fn backspace_removes_last_char_from_query() {
        let mut app = App::new(make_prompts());
        app.handle_event(&key(KeyCode::Char('p')));
        app.handle_event(&key(KeyCode::Char('e')));
        app.handle_event(&key(KeyCode::Backspace));
        assert_eq!(app.query, "p");
    }

    #[test]
    fn backspace_on_empty_query_is_noop() {
        let mut app = App::new(make_prompts());
        app.handle_event(&key(KeyCode::Backspace));
        assert_eq!(app.query, "");
    }

    #[test]
    fn empty_query_shows_all_prompts() {
        let mut app = App::new(make_prompts());
        app.handle_event(&key(KeyCode::Char('z')));
        app.handle_event(&key(KeyCode::Backspace));
        assert_eq!(app.filtered.len(), 3);
    }

    // ── fuzzy filter ────────────────────────────────────────────────────────

    #[test]
    fn typing_filters_prompts_by_query() {
        let mut app = App::new(make_prompts());
        for c in "write".chars() {
            app.handle_event(&key(KeyCode::Char(c)));
        }
        let matched_ids: Vec<&str> = app
            .filtered
            .iter()
            .map(|&i| app.prompts[i].id.as_str())
            .collect();
        assert!(matched_ids.contains(&"write-tests"));
        assert!(!app.filtered.is_empty());
    }

    #[test]
    fn clearing_query_restores_full_list() {
        let mut app = App::new(make_prompts());
        app.handle_event(&key(KeyCode::Char('p')));
        // "p" matches only "perf-review" → filtered.len() == 1
        assert_eq!(app.filtered.len(), 1);
        app.handle_event(&key(KeyCode::Backspace));
        assert_eq!(app.filtered.len(), 3);
    }

    // ── navigation ──────────────────────────────────────────────────────────

    #[test]
    fn down_key_increments_selection() {
        let mut app = App::new(make_prompts());
        app.handle_event(&key(KeyCode::Down));
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn up_key_at_start_is_noop() {
        let mut app = App::new(make_prompts());
        app.handle_event(&key(KeyCode::Up));
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn up_key_decrements_selection() {
        let mut app = App::new(make_prompts());
        app.handle_event(&key(KeyCode::Down));
        app.handle_event(&key(KeyCode::Down));
        app.handle_event(&key(KeyCode::Up));
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn selection_clamps_at_last_item() {
        let mut app = App::new(make_prompts()); // 3 prompts → max index 2
        for _ in 0..10 {
            app.handle_event(&key(KeyCode::Down));
        }
        assert_eq!(app.selected, 2);
    }

    #[test]
    fn selection_clamps_when_filter_narrows() {
        let mut app = App::new(make_prompts());
        for _ in 0..10 {
            app.handle_event(&key(KeyCode::Down));
        }
        assert_eq!(app.selected, 2);
        for c in "write".chars() {
            app.handle_event(&key(KeyCode::Char(c)));
        }
        if !app.filtered.is_empty() {
            assert!(app.selected < app.filtered.len());
        }
    }

    // ── mouse scroll ────────────────────────────────────────────────────────

    #[test]
    fn mouse_scroll_down_increments_selection() {
        let mut app = App::new(make_prompts());
        app.handle_event(&mouse_scroll(MouseEventKind::ScrollDown));
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn mouse_scroll_up_at_start_is_noop() {
        let mut app = App::new(make_prompts());
        app.handle_event(&mouse_scroll(MouseEventKind::ScrollUp));
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn mouse_scroll_up_decrements_selection() {
        let mut app = App::new(make_prompts());
        app.handle_event(&mouse_scroll(MouseEventKind::ScrollDown));
        app.handle_event(&mouse_scroll(MouseEventKind::ScrollUp));
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn mouse_scroll_down_clamps_at_last_item() {
        let mut app = App::new(make_prompts());
        for _ in 0..10 {
            app.handle_event(&mouse_scroll(MouseEventKind::ScrollDown));
        }
        assert_eq!(app.selected, 2);
    }
}
