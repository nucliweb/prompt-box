use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
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
    pub list_area: Rect,
    pub copied: Option<(String, String)>, // (title, prompt_text) set on Enter
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
            list_area: Rect::default(),
            copied: None,
        }
    }

    pub fn selected_prompt(&self) -> Option<&Prompt> {
        self.filtered.get(self.selected).map(|&idx| &self.prompts[idx])
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
                KeyCode::Enter => {
                    if let Some(p) = self.selected_prompt() {
                        self.copied = Some((p.title.clone(), p.prompt.clone()));
                        self.running = false;
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
                MouseEventKind::Down(MouseButton::Left) => {
                    let col = mouse_event.column;
                    let row = mouse_event.row;
                    let a = self.list_area;
                    // skip top/bottom borders
                    let within = col >= a.x
                        && col < a.x + a.width
                        && row > a.y
                        && row < a.y + a.height.saturating_sub(1);
                    if within {
                        let item_idx = (row - a.y - 1) as usize;
                        if item_idx < self.filtered.len() {
                            self.selected = item_idx;
                        }
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

fn render(frame: &mut ratatui::Frame, app: &mut App) {
    let area = frame.area();

    // Outer vertical: search | main | status
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    // Main area: list (50%) | preview (50%)
    let main_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(outer[1]);

    // Store list bounds so handle_event can map clicks to items
    app.list_area = main_cols[0];

    // Search box
    frame.render_widget(
        Paragraph::new(format!("{}█", app.query))
            .block(Block::default().borders(Borders::ALL).title("Search")),
        outer[0],
    );

    // Prompt list
    let items: Vec<ListItem> = app
        .filtered
        .iter()
        .map(|&idx| {
            let p = &app.prompts[idx];
            ListItem::new(Line::from(Span::raw(format!(
                "[{}] {} — {}",
                p.category, p.id, p.title
            ))))
        })
        .collect();

    let mut list_state = ListState::default();
    if !app.filtered.is_empty() {
        list_state.select(Some(app.selected));
    }
    frame.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Prompts"))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .highlight_symbol("▶ "),
        main_cols[0],
        &mut list_state,
    );

    // Preview pane
    let sep_len = main_cols[1].width.saturating_sub(2) as usize;
    let preview_lines: Vec<Line> = match app.selected_prompt() {
        None => vec![Line::from("No prompt selected.")],
        Some(p) => {
            let mut lines = vec![
                Line::from(Span::styled(
                    p.title.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(format!("Tags: {}", p.tags.join(", "))),
                Line::from(format!("Description: {}", p.description)),
                Line::from(Span::styled(
                    "─".repeat(sep_len),
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
            ];
            for text_line in p.prompt.lines() {
                lines.push(Line::from(text_line.to_string()));
            }
            lines
        }
    };
    frame.render_widget(
        Paragraph::new(preview_lines)
            .block(Block::default().borders(Borders::ALL).title("Preview"))
            .wrap(Wrap { trim: false }),
        main_cols[1],
    );

    // Status bar
    frame.render_widget(
        Paragraph::new("[↑↓/scroll] Navigate  [Enter] Copy & Exit  [Esc/q] Quit")
            .style(Style::default().fg(Color::DarkGray)),
        outer[2],
    );
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
    let guard = TerminalGuard::setup()?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let prompts = crate::storage::load_prompts().unwrap_or_default();
    let mut app = App::new(prompts);

    while app.running {
        terminal.draw(|frame| {
            render(frame, &mut app);
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            app.handle_event(&event::read()?);
        }
    }

    // Restore terminal before any stdout output so the message appears on the
    // main screen, not inside the (soon-to-be-cleared) alternate screen buffer.
    drop(terminal);
    drop(guard);

    if let Some((title, prompt_text)) = app.copied {
        arboard::Clipboard::new()
            .and_then(|mut cb| cb.set_text(prompt_text))
            .map_err(|e| anyhow::anyhow!("clipboard error: {}", e))?;
        println!("Copied: {}", title);
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

    fn mouse_click(column: u16, row: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
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
        let mut app = App::new(make_prompts());
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

    // ── selected_prompt ─────────────────────────────────────────────────────

    #[test]
    fn selected_prompt_returns_first_prompt_by_default() {
        let app = App::new(make_prompts());
        let p = app.selected_prompt().unwrap();
        assert_eq!(p.id, "perf-review");
    }

    #[test]
    fn selected_prompt_returns_none_for_empty_list() {
        let app = App::new(vec![]);
        assert!(app.selected_prompt().is_none());
    }

    #[test]
    fn selected_prompt_updates_after_navigation() {
        let mut app = App::new(make_prompts());
        app.handle_event(&key(KeyCode::Down));
        let p = app.selected_prompt().unwrap();
        assert_eq!(p.id, "code-review");
    }

    #[test]
    fn selected_prompt_reflects_filtered_order() {
        let mut app = App::new(make_prompts());
        for c in "write".chars() {
            app.handle_event(&key(KeyCode::Char(c)));
        }
        // "write" should rank "write-tests" first
        let p = app.selected_prompt().unwrap();
        assert_eq!(p.id, "write-tests");
    }

    // ── mouse click selection ───────────────────────────────────────────────

    // list_area: x=0, y=3, width=40, height=20
    // top border at row 3; first item at row 4; bottom border at row 22
    fn list_area() -> Rect {
        Rect { x: 0, y: 3, width: 40, height: 20 }
    }

    #[test]
    fn mouse_click_selects_first_item() {
        let mut app = App::new(make_prompts());
        app.list_area = list_area();
        app.handle_event(&mouse_click(5, 4)); // row 4 = item 0
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn mouse_click_selects_second_item() {
        let mut app = App::new(make_prompts());
        app.list_area = list_area();
        app.handle_event(&mouse_click(5, 5)); // row 5 = item 1
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn mouse_click_on_top_border_is_noop() {
        let mut app = App::new(make_prompts());
        app.list_area = list_area();
        app.handle_event(&mouse_click(5, 3)); // row 3 = top border
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn mouse_click_outside_list_is_noop() {
        let mut app = App::new(make_prompts());
        app.list_area = list_area();
        app.handle_event(&key(KeyCode::Down)); // move to item 1
        app.handle_event(&mouse_click(50, 5)); // col 50 is outside width=40
        assert_eq!(app.selected, 1); // unchanged
    }

    #[test]
    fn mouse_click_beyond_items_is_noop() {
        let mut app = App::new(make_prompts()); // 3 items
        app.list_area = list_area();
        app.handle_event(&mouse_click(5, 10)); // row 10 = item index 6, beyond 3 items
        assert_eq!(app.selected, 0); // unchanged
    }

    // ── Enter key: copy intent ───────────────────────────────────────────────

    #[test]
    fn enter_with_selection_sets_copied() {
        let mut app = App::new(make_prompts());
        app.handle_event(&key(KeyCode::Enter));
        let (title, prompt) = app.copied.as_ref().unwrap();
        assert_eq!(title, "Performance Review");
        assert_eq!(prompt, "Review performance...");
    }

    #[test]
    fn enter_with_selection_stops_running() {
        let mut app = App::new(make_prompts());
        app.handle_event(&key(KeyCode::Enter));
        assert!(!app.running);
    }

    #[test]
    fn enter_on_empty_list_is_noop() {
        let mut app = App::new(vec![]);
        app.handle_event(&key(KeyCode::Enter));
        assert!(app.copied.is_none());
        assert!(app.running);
    }

    #[test]
    fn enter_on_empty_filtered_list_is_noop() {
        let mut app = App::new(make_prompts());
        for c in "xyzxyz".chars() {
            app.handle_event(&key(KeyCode::Char(c)));
        }
        assert!(app.filtered.is_empty());
        app.handle_event(&key(KeyCode::Enter));
        assert!(app.copied.is_none());
        assert!(app.running);
    }

    #[test]
    fn enter_copies_the_selected_prompt() {
        let mut app = App::new(make_prompts());
        app.handle_event(&key(KeyCode::Down)); // select "code-review"
        app.handle_event(&key(KeyCode::Enter));
        let (title, prompt) = app.copied.as_ref().unwrap();
        assert_eq!(title, "Code Review");
        assert_eq!(prompt, "Review the code...");
    }
}
