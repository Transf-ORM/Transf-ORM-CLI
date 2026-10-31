//! Full-screen interface of the interactive mode, in the style of Claude Code.
//!
//! The session runs on the terminal's alternate screen: what the terminal
//! showed is hidden while it runs and comes back untouched on exit, with no
//! trace of the session. The screen holds the conversation (the welcome box,
//! then each command and its output), which scrolls, and at the bottom the
//! input box with the command menu or the status line, or the interactive
//! box ([`Panel`]) a command opened, such as `/help`.
//!
//! Commands write into the conversation through a [`Console`]: printing to
//! stdout directly would corrupt the screen.

use std::io;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{
    self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    Event, KeyCode, KeyEvent, KeyEventKind, MouseEventKind,
};
use ratatui::crossterm::execute;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Padding, Paragraph, Widget};
use ratatui::{DefaultTerminal, Frame};
use transf_orm_core::config::GlobalConfig;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::editor::{Action, Editor, MenuItem};
use super::{handle_line, Flow, Startup};
use crate::ui::theme::{ACCENT, ERROR};
use crate::ui::{self, Panel, PanelEvent};
use crate::Console;

const PROMPT: &str = "> ";
const PLACEHOLDER: &str = "Type a command, or / for the list";
/// Most menu entries shown at once; the menu scrolls beyond.
const MENU_ROWS: usize = 5;
/// Lines scrolled by one notch of the mouse wheel.
const WHEEL_LINES: usize = 3;

/// Runs the interactive session on the alternate screen until the user
/// quits, then restores the terminal.
///
/// # Errors
///
/// Returns an error when the terminal cannot be set up, drawn or read.
pub(super) fn run(
    config: &GlobalConfig,
    startup: &Startup,
    items: Vec<MenuItem>,
) -> io::Result<()> {
    // Raw mode + alternate screen, with a panic hook that restores them.
    let mut terminal = ratatui::try_init()?;
    let _screen = Screen::enable()?;
    App::new(startup, items).run(config, &mut terminal)
}

/// Mouse wheel and bracketed paste for the session; on drop, restores the
/// terminal (also when the session fails).
struct Screen;

impl Screen {
    fn enable() -> io::Result<Self> {
        let screen = Screen;
        execute!(io::stdout(), EnableMouseCapture, EnableBracketedPaste)?;
        Ok(screen)
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), DisableMouseCapture, DisableBracketedPaste);
        ratatui::restore();
    }
}

struct App {
    banner: Vec<Line<'static>>,
    /// One block of lines per submitted command: its echo, then its output.
    conversation: Vec<Vec<Line<'static>>>,
    status: String,
    items: Vec<MenuItem>,
    history: Vec<String>,
    editor: Editor,
    /// Interactive box opened by a command; it gets the keys while open.
    panel: Option<Box<dyn Panel>>,
    /// Lines scrolled up from the bottom of the conversation; 0 follows the
    /// latest output.
    scroll: usize,
    /// Height of the conversation area at the last draw, for PgUp / PgDn.
    page: usize,
}

impl App {
    fn new(startup: &Startup, items: Vec<MenuItem>) -> Self {
        let status = match startup.detected.first() {
            Some(first) => format!("{} detected", first.orm.display_name),
            None => "no ORM detected".to_owned(),
        };
        Self {
            banner: banner(startup),
            conversation: Vec::new(),
            status,
            items,
            history: Vec::new(),
            editor: Editor::default(),
            panel: None,
            scroll: 0,
            page: 0,
        }
    }

    fn run(&mut self, config: &GlobalConfig, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;
            match event::read()? {
                Event::Key(key) if key.kind != KeyEventKind::Release => {
                    let keep_going = match key.code {
                        KeyCode::PageUp => {
                            self.scroll_up(self.page / 2);
                            continue;
                        }
                        KeyCode::PageDown => {
                            self.scroll_down(self.page / 2);
                            continue;
                        }
                        _ if self.panel.is_some() => self.handle_panel_key(config, key),
                        _ => match self.editor.handle_key(key, &self.items, &self.history) {
                            Action::Redraw => continue,
                            Action::Exit => false,
                            Action::Submit(line) => self.submit(config, line),
                        },
                    };
                    if !keep_going {
                        return Ok(());
                    }
                    // Repaint everything: something may have written to the
                    // terminal directly while a command ran (e.g. `--debug` logs).
                    terminal.clear()?;
                }
                Event::Paste(text) if self.panel.is_none() => {
                    self.editor.insert(&text.replace(['\r', '\n'], " "));
                }
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollUp => self.scroll_up(WHEEL_LINES),
                    MouseEventKind::ScrollDown => self.scroll_down(WHEEL_LINES),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    /// Runs a submitted line and adds it, with its output, to the
    /// conversation.
    ///
    /// # Returns
    ///
    /// `false` when the session must end.
    fn submit(&mut self, config: &GlobalConfig, line: String) -> bool {
        if self.history.last() != Some(&line) {
            self.history.push(line.clone());
        }
        self.editor = Editor::default();
        self.scroll = 0;

        let mut capture = Capture::default();
        capture.lines.push(Line::from(vec![
            Span::styled(PROMPT, Style::new().fg(ACCENT)),
            Span::styled(line.clone(), Style::new().bold()),
        ]));
        match handle_line(config, &line, &mut capture) {
            Flow::Exit => return false,
            Flow::Clear => self.conversation.clear(),
            Flow::Open(panel) => {
                self.conversation.push(capture.lines);
                self.panel = Some(panel);
            }
            Flow::Continue => self.conversation.push(capture.lines),
        }
        true
    }

    /// Sends a key to the open panel and applies what it asks for.
    ///
    /// # Returns
    ///
    /// `false` when the session must end.
    fn handle_panel_key(&mut self, config: &GlobalConfig, key: KeyEvent) -> bool {
        let Some(panel) = self.panel.as_mut() else {
            return true;
        };
        match panel.handle_key(key) {
            PanelEvent::Continue => {}
            PanelEvent::Close => self.close_panel(),
            PanelEvent::Run(line) => {
                self.panel = None;
                return self.submit(config, line);
            }
            PanelEvent::Fill(text) => {
                self.panel = None;
                self.editor.set_input(text);
            }
        }
        true
    }

    /// Closes the panel, noting it under the command that opened it.
    fn close_panel(&mut self) {
        if let Some(panel) = self.panel.take() {
            let note = Line::styled(
                format!("  ⎿ {} closed", panel.title()),
                Style::new().dark_gray(),
            );
            match self.conversation.last_mut() {
                Some(block) => block.push(note),
                None => self.conversation.push(vec![note]),
            }
        }
    }

    fn scroll_up(&mut self, lines: usize) {
        // Clamped to the conversation height when drawing.
        self.scroll = self.scroll.saturating_add(lines.max(1));
    }

    fn scroll_down(&mut self, lines: usize) {
        self.scroll = self.scroll.saturating_sub(lines.max(1));
    }

    fn render(&mut self, frame: &mut Frame) {
        // The bottom holds the open panel, or the input box with the menu or
        // status line below it.
        let bottom = match &self.panel {
            Some(panel) => ui::height(panel.as_ref(), frame.area().height / 2),
            None => {
                3 + self
                    .editor
                    .suggestions(&self.items)
                    .len()
                    .clamp(1, MENU_ROWS) as u16
            }
        };
        let [conversation, _gap, bottom] = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(bottom),
        ])
        .areas(frame.area());

        self.page = conversation.height.into();
        self.render_conversation(frame.buffer_mut(), conversation);
        if let Some(panel) = self.panel.as_mut() {
            ui::draw(panel.as_mut(), bottom, frame.buffer_mut());
            return;
        }

        let [input, below] =
            Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(bottom);
        self.render_input(frame, input);
        let suggestions = self.editor.suggestions(&self.items);
        if suggestions.is_empty() {
            self.render_status(frame, below);
        } else {
            render_menu(frame, &suggestions, self.editor.selected, below);
        }
    }

    /// Draws the banner and the conversation blocks, separated by blank
    /// lines, scrolled so the latest output sits at the bottom.
    fn render_conversation(&mut self, buf: &mut Buffer, area: Rect) {
        // The banner is drawn off screen, then its visible rows are copied.
        let banner_height = self.banner.len() as u16 + 2;
        let mut banner = Buffer::empty(Rect::new(0, 0, area.width, banner_height));
        Paragraph::new(self.banner.clone())
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(ACCENT))
                    .padding(Padding::horizontal(1)),
            )
            .render(banner.area, &mut banner);

        // One entry per screen row, so the height is known exactly.
        let mut rows = Vec::new();
        for block in &self.conversation {
            rows.push(Line::default());
            for line in block {
                rows.extend(wrap(line, area.width));
            }
        }

        let height = usize::from(banner_height) + rows.len();
        let view = usize::from(area.height);
        self.scroll = self.scroll.min(height.saturating_sub(view));
        let top = height.saturating_sub(view + self.scroll);

        for row in top..height.min(top + view) {
            let y = area.y + (row - top) as u16;
            match row.checked_sub(banner_height.into()) {
                Some(index) => {
                    buf.set_line(area.x, y, &rows[index], area.width);
                }
                None => {
                    for x in 0..area.width {
                        if let Some(cell) = buf.cell_mut((area.x + x, y)) {
                            *cell = banner[(x, row as u16)].clone();
                        }
                    }
                }
            }
        }
    }

    fn render_input(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().dark_gray())
            .padding(Padding::horizontal(1));
        let inner = block.inner(area);
        let prompt = Span::styled(PROMPT, Style::new().fg(ACCENT).bold());
        let text = if self.editor.input.is_empty() {
            Line::from(vec![
                prompt,
                Span::styled(PLACEHOLDER, Style::new().dark_gray()),
            ])
        } else {
            Line::from(vec![prompt, Span::raw(self.editor.input.as_str())])
        };
        // Scroll horizontally so the cursor stays visible in a long line.
        let cursor_col = (PROMPT.width() + self.editor.input[..self.editor.cursor].width()) as u16;
        let offset = (cursor_col + 1).saturating_sub(inner.width);
        frame.render_widget(Paragraph::new(text).scroll((0, offset)).block(block), area);
        frame.set_cursor_position((inner.x + cursor_col - offset, inner.y));
    }

    fn render_status(&self, frame: &mut Frame, area: Rect) {
        let hint = if self.editor.exit_armed {
            Span::styled("  Press Ctrl-C again to exit", Style::new().fg(ACCENT))
        } else if self.scroll > 0 {
            Span::styled(
                "  ↓ PgDn or scroll down for the latest output",
                Style::new().fg(ACCENT),
            )
        } else {
            Span::styled(
                "  / for commands · ↑↓ history · PgUp/PgDn scroll · Ctrl-D to exit",
                Style::new().dark_gray(),
            )
        };
        let status = format!("{}  ", self.status);
        // The detected ORM goes on the right when it does not overlap the hint.
        if hint.width() + status.width() < usize::from(area.width) {
            frame.render_widget(
                Line::styled(status, Style::new().dark_gray()).right_aligned(),
                area,
            );
        }
        frame.render_widget(hint, area);
    }
}

fn render_menu(frame: &mut Frame, items: &[&MenuItem], selected: usize, area: Rect) {
    let name_width = items
        .iter()
        .map(|item| item.name.width())
        .max()
        .unwrap_or(0);
    // Keep the selection in view when there are more items than rows.
    let first = selected.saturating_sub(MENU_ROWS - 1);
    let lines = items
        .iter()
        .enumerate()
        .skip(first)
        .take(MENU_ROWS)
        .map(|(index, item)| {
            let name = format!("{:<name_width$}", item.name);
            if index == selected {
                Line::from(vec![
                    Span::styled("  › ", Style::new().fg(ACCENT)),
                    Span::styled(name, Style::new().fg(ACCENT).bold()),
                    Span::raw("   "),
                    Span::raw(item.about.clone()),
                ])
            } else {
                Line::from(vec![
                    Span::raw("    "),
                    Span::raw(name),
                    Span::raw("   "),
                    Span::styled(item.about.clone(), Style::new().dark_gray()),
                ])
            }
        })
        .collect::<Vec<_>>();
    frame.render_widget(Paragraph::new(lines), area);
}

/// Splits `line` into rows of at most `width` columns, cutting between
/// characters and keeping the styles.
fn wrap(line: &Line<'static>, width: u16) -> Vec<Line<'static>> {
    let width = usize::from(width.max(1));
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut row_width = 0;
    for span in &line.spans {
        let mut chunk = String::new();
        for c in span.content.chars() {
            let c_width = c.width().unwrap_or(0);
            if row_width + c_width > width && row_width > 0 {
                if !chunk.is_empty() {
                    row.push(Span::styled(std::mem::take(&mut chunk), span.style));
                }
                rows.push(Line::from(std::mem::take(&mut row)).style(line.style));
                row_width = 0;
            }
            chunk.push(c);
            row_width += c_width;
        }
        if !chunk.is_empty() {
            row.push(Span::styled(chunk, span.style));
        }
    }
    rows.push(Line::from(row).style(line.style));
    rows
}

/// The welcome box: version, current directory and detected ORM.
fn banner(startup: &Startup) -> Vec<Line<'static>> {
    let label = |text: &'static str| Span::styled(text, Style::new().dark_gray());
    let orm = match startup.detected.first() {
        Some(first) => Line::from(vec![
            label("  ORM  "),
            Span::styled(first.orm.display_name, Style::new().bold()),
            Span::raw(" · "),
            Span::raw(Startup::files_summary(first, startup.detected.len() - 1)),
        ]),
        None => Line::from(vec![
            label("  ORM  "),
            Span::raw("none detected"),
            Span::styled(
                " · use --from to choose the source ORM",
                Style::new().dark_gray(),
            ),
        ]),
    };
    vec![
        Line::from(vec![
            Span::styled("✻ ", Style::new().fg(ACCENT)),
            Span::styled("Transf-ORM", Style::new().bold()),
            Span::styled(
                format!(" {}", env!("CARGO_PKG_VERSION")),
                Style::new().dark_gray(),
            ),
        ]),
        Line::default(),
        Line::from(vec![
            label("  cwd  "),
            Span::raw(home_relative(&startup.cwd)),
        ]),
        orm,
    ]
}

/// `path` with the home directory shown as `~`.
fn home_relative(path: &std::path::Path) -> String {
    let rest = std::env::var_os("HOME").and_then(|home| path.strip_prefix(home).ok());
    match rest {
        Some(rest) if rest.as_os_str().is_empty() => "~".to_owned(),
        Some(rest) => format!("~/{}", rest.display()),
        None => path.display().to_string(),
    }
}

/// [`Console`] that collects a command's output as conversation lines.
#[derive(Default)]
struct Capture {
    lines: Vec<Line<'static>>,
}

impl Capture {
    fn push(&mut self, text: &str, style: impl Fn(&str) -> Style) {
        for line in text.replace('\t', "    ").lines() {
            self.lines.push(Line::styled(line.to_owned(), style(line)));
        }
    }
}

impl Console for Capture {
    fn out(&mut self, text: &str) {
        self.push(text, |_| Style::new());
    }

    fn err(&mut self, text: &str) {
        self.push(text, |line| {
            if line.starts_with("error") {
                Style::new().fg(ERROR)
            } else {
                Style::new().dark_gray()
            }
        });
    }

    fn confirm(&mut self, question: &str) -> bool {
        self.err(&format!("{question}: cannot be confirmed here"));
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn app(blocks: usize) -> App {
        let startup = Startup {
            cwd: "/tmp".into(),
            detected: Vec::new(),
        };
        let mut app = App::new(&startup, Vec::new());
        for block in 0..blocks {
            let mut capture = Capture::default();
            capture.out(&format!("block {block}"));
            app.conversation.push(capture.lines);
        }
        app
    }

    /// The text of each row of the screen after drawing `app`.
    fn screen(app: &mut App, width: u16, height: u16) -> Vec<String> {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| app.render(frame)).unwrap();
        let buffer = terminal.backend().buffer();
        (0..height)
            .map(|y| (0..width).map(|x| buffer[(x, y)].symbol()).collect())
            .collect()
    }

    #[test]
    fn short_conversation_starts_at_the_top() {
        let rows = screen(&mut app(1), 60, 20);
        assert!(rows[1].contains("Transf-ORM"), "{rows:#?}");
        assert!(rows[7].contains("block 0"), "{rows:#?}");
        assert!(rows[17].contains(PLACEHOLDER), "{rows:#?}");
    }

    #[test]
    fn long_conversation_shows_the_latest_output() {
        let mut app = app(30);
        let rows = screen(&mut app, 60, 20);
        assert!(rows[14].contains("block 29"), "{rows:#?}");
        assert!(!rows.iter().any(|row| row.contains("Transf-ORM")));
    }

    #[test]
    fn scrolling_up_is_clamped_to_the_banner() {
        let mut app = app(30);
        app.scroll_up(10_000);
        let rows = screen(&mut app, 60, 20);
        assert!(rows[1].contains("Transf-ORM"), "{rows:#?}");
        assert!(rows[19].contains("PgDn"), "{rows:#?}");
        app.scroll_down(10_000);
        assert_eq!(app.scroll, 0);
    }

    #[test]
    fn long_lines_wrap_to_the_screen_width() {
        let mut app = app(0);
        let mut capture = Capture::default();
        capture.out(&"a".repeat(25));
        app.conversation.push(capture.lines);
        let rows = screen(&mut app, 10, 20);
        // Banner (6 rows), blank line, then 10 + 10 + 5 characters.
        assert_eq!(rows[7], "aaaaaaaaaa");
        assert_eq!(rows[8], "aaaaaaaaaa");
        assert_eq!(rows[9].trim_end(), "aaaaa");
    }

    #[test]
    fn wrap_keeps_styles_and_wide_characters_whole() {
        let line = Line::from(vec![
            Span::styled("> ", Style::new().fg(ACCENT)),
            Span::raw("日本語"),
        ]);
        let rows = wrap(&line, 4);
        let text = |row: &Line| {
            row.spans
                .iter()
                .map(|s| s.content.as_ref())
                .collect::<String>()
        };
        assert_eq!(rows.iter().map(text).collect::<Vec<_>>(), ["> 日", "本語"]);
        assert_eq!(rows[0].spans[0].style.fg, Some(ACCENT));

        assert_eq!(wrap(&Line::default(), 10).len(), 1);
    }

    #[test]
    fn errors_are_styled_and_split_into_lines() {
        let mut capture = Capture::default();
        capture.err("error: boom\nUse --from");
        assert_eq!(capture.lines.len(), 2);
        assert_eq!(capture.lines[0].style.fg, Some(ERROR));
        assert!(!capture.confirm("Detected: X"));
    }

    fn config() -> GlobalConfig {
        GlobalConfig {
            debug: None,
            verbose: false,
            perf: false,
            tts: false,
            daltonien: false,
        }
    }

    fn key(app: &mut App, code: KeyCode) -> bool {
        app.handle_panel_key(&config(), KeyEvent::from(code))
    }

    #[test]
    fn help_opens_a_box_in_place_of_the_input() {
        let mut app = app(0);
        assert!(app.submit(&config(), "/help".into()));
        let rows = screen(&mut app, 70, 24);
        assert!(rows.iter().any(|row| row.contains("╭ Help ")), "{rows:#?}");
        assert!(rows.iter().any(|row| row.contains("› /help")), "{rows:#?}");
        assert!(rows[23].contains("Enter select"), "{rows:#?}");
        assert!(!rows.iter().any(|row| row.contains(PLACEHOLDER)));
    }

    #[test]
    fn escape_closes_the_box_with_a_note() {
        let mut app = app(0);
        app.submit(&config(), "/help".into());
        assert!(key(&mut app, KeyCode::Esc));
        assert!(app.panel.is_none());
        let rows = screen(&mut app, 70, 24);
        assert!(
            rows.iter().any(|row| row.contains("⎿ Help closed")),
            "{rows:#?}"
        );
        assert!(rows.iter().any(|row| row.contains(PLACEHOLDER)));
    }

    #[test]
    fn enter_in_the_box_runs_or_fills_the_command() {
        let mut app = app(2);
        app.submit(&config(), "/help".into());
        key(&mut app, KeyCode::Down);
        assert!(key(&mut app, KeyCode::Enter), "/clear keeps the session");
        assert!(app.conversation.is_empty());

        app.submit(&config(), "/help".into());
        key(&mut app, KeyCode::End);
        key(&mut app, KeyCode::Enter);
        assert!(app.panel.is_none());
        assert_eq!(app.editor.input, "convert ");

        app.submit(&config(), "/help".into());
        key(&mut app, KeyCode::Down);
        key(&mut app, KeyCode::Down);
        assert!(!key(&mut app, KeyCode::Enter), "/exit ends the session");
    }

    #[test]
    fn renders_at_any_size() {
        for (width, height) in [(1, 1), (10, 4), (40, 10), (120, 40)] {
            let mut app = app(3);
            app.editor.insert(&"x".repeat(200));
            screen(&mut app, width, height);
            app.submit(&config(), "/help".into());
            screen(&mut app, width, height);
        }
    }
}
