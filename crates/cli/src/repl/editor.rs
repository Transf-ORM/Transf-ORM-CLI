//! The line being typed in the interactive interface: cursor, history,
//! command menu and key bindings. Independent of the terminal so it can be
//! unit tested.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// An entry of the command menu.
pub(super) struct MenuItem {
    pub name: String,
    pub about: String,
    /// CLI commands need arguments, so choosing one completes the line
    /// instead of running it.
    pub takes_args: bool,
}

/// Result of a key press.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Action {
    Redraw,
    Submit(String),
    Exit,
}

/// State of the line being typed. Independent of the terminal so it can be
/// unit tested.
#[derive(Default)]
pub(super) struct Editor {
    pub(super) input: String,
    /// Byte offset of the cursor in `input`, always on a char boundary.
    pub(super) cursor: usize,
    /// Index in the history while browsing it with ↑/↓.
    history_pos: Option<usize>,
    /// Line being typed before browsing the history, restored by ↓.
    draft: String,
    /// Selected entry of the command menu.
    pub(super) selected: usize,
    /// Esc closed the menu; it reopens on the next edit.
    menu_dismissed: bool,
    /// Ctrl-C on an empty line: a second one exits.
    pub(super) exit_armed: bool,
}

impl Editor {
    pub(super) fn handle_key(
        &mut self,
        key: KeyEvent,
        items: &[MenuItem],
        history: &[String],
    ) -> Action {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let was_armed = std::mem::take(&mut self.exit_armed);
        let suggestions = self.suggestions(items);

        match key.code {
            KeyCode::Char('c') if ctrl => {
                if !self.input.is_empty() {
                    self.set_input(String::new());
                } else if was_armed {
                    return Action::Exit;
                } else {
                    self.exit_armed = true;
                }
            }
            KeyCode::Char('d') if ctrl && self.input.is_empty() => return Action::Exit,
            KeyCode::Char('d') if ctrl => self.delete_forward(),
            KeyCode::Char('a') if ctrl => self.cursor = 0,
            KeyCode::Char('e') if ctrl => self.cursor = self.input.len(),
            KeyCode::Char('u') if ctrl => {
                self.input.drain(..self.cursor);
                self.cursor = 0;
                self.edited();
            }
            KeyCode::Char('w') if ctrl => self.delete_word(),
            KeyCode::Char(c) if !ctrl => self.insert(c.encode_utf8(&mut [0; 4])),
            KeyCode::Enter => {
                if let Some(item) = suggestions.get(self.selected) {
                    if !item.takes_args {
                        return Action::Submit(item.name.clone());
                    }
                    if self.input != item.name {
                        self.complete(item);
                        return Action::Redraw;
                    }
                }
                let line = self.input.trim();
                if !line.is_empty() {
                    return Action::Submit(line.to_owned());
                }
            }
            KeyCode::Tab => {
                if let Some(item) = suggestions.get(self.selected) {
                    self.complete(item);
                }
            }
            KeyCode::Esc => self.menu_dismissed = true,
            KeyCode::Up if !suggestions.is_empty() => {
                self.selected = (self.selected + suggestions.len() - 1) % suggestions.len();
            }
            KeyCode::Down if !suggestions.is_empty() => {
                self.selected = (self.selected + 1) % suggestions.len();
            }
            KeyCode::Up => self.history_prev(history),
            KeyCode::Down => self.history_next(history),
            KeyCode::Left => self.cursor = self.prev_boundary(),
            KeyCode::Right => self.cursor = self.next_boundary(),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.input.len(),
            KeyCode::Backspace if self.cursor > 0 => {
                let start = self.prev_boundary();
                self.input.drain(start..self.cursor);
                self.cursor = start;
                self.edited();
            }
            KeyCode::Delete => self.delete_forward(),
            _ => {}
        }
        Action::Redraw
    }

    /// Menu entries matching the line: session commands while typing
    /// `/name`, CLI commands while typing their first word.
    pub(super) fn suggestions<'a>(&self, items: &'a [MenuItem]) -> Vec<&'a MenuItem> {
        if self.menu_dismissed || self.input.is_empty() || self.input.contains(char::is_whitespace)
        {
            return Vec::new();
        }
        let session = self.input.starts_with('/');
        items
            .iter()
            .filter(|item| item.takes_args != session && item.name.starts_with(&self.input))
            .collect()
    }

    pub(super) fn insert(&mut self, text: &str) {
        self.input.insert_str(self.cursor, text);
        self.cursor += text.len();
        self.edited();
    }

    fn complete(&mut self, item: &MenuItem) {
        let mut line = item.name.clone();
        if item.takes_args {
            line.push(' ');
        }
        self.set_input(line);
    }

    pub(super) fn set_input(&mut self, input: String) {
        self.cursor = input.len();
        self.input = input;
        self.edited();
    }

    /// Resets the menu after any change of the line.
    fn edited(&mut self) {
        self.selected = 0;
        self.menu_dismissed = false;
    }

    fn delete_forward(&mut self) {
        if self.cursor < self.input.len() {
            let end = self.next_boundary();
            self.input.drain(self.cursor..end);
            self.edited();
        }
    }

    fn delete_word(&mut self) {
        let before = self.input[..self.cursor].trim_end();
        let start = before.rfind(char::is_whitespace).map_or(0, |i| i + 1);
        self.input.drain(start..self.cursor);
        self.cursor = start;
        self.edited();
    }

    fn prev_boundary(&self) -> usize {
        self.input[..self.cursor]
            .char_indices()
            .next_back()
            .map_or(0, |(i, _)| i)
    }

    fn next_boundary(&self) -> usize {
        self.input[self.cursor..]
            .chars()
            .next()
            .map_or(self.cursor, |c| self.cursor + c.len_utf8())
    }

    fn history_prev(&mut self, history: &[String]) {
        let pos = match self.history_pos {
            _ if history.is_empty() => return,
            None => {
                self.draft = self.input.clone();
                history.len() - 1
            }
            Some(pos) => pos.saturating_sub(1),
        };
        self.history_pos = Some(pos);
        self.set_input(history[pos].clone());
    }

    fn history_next(&mut self, history: &[String]) {
        match self.history_pos {
            None => {}
            Some(pos) if pos + 1 < history.len() => {
                self.history_pos = Some(pos + 1);
                self.set_input(history[pos + 1].clone());
            }
            Some(_) => {
                self.history_pos = None;
                let draft = std::mem::take(&mut self.draft);
                self.set_input(draft);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> Vec<MenuItem> {
        let item = |name: &str, takes_args| MenuItem {
            name: name.to_owned(),
            about: format!("about {name}"),
            takes_args,
        };
        vec![
            item("/help", false),
            item("/exit", false),
            item("convert", true),
        ]
    }

    fn press(editor: &mut Editor, code: KeyCode, history: &[String]) -> Action {
        editor.handle_key(KeyEvent::from(code), &items(), history)
    }

    fn ctrl(editor: &mut Editor, c: char) -> Action {
        editor.handle_key(
            KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL),
            &items(),
            &[],
        )
    }

    fn typed(text: &str) -> Editor {
        let mut editor = Editor::default();
        editor.insert(text);
        editor
    }

    fn names(editor: &Editor) -> Vec<String> {
        editor
            .suggestions(&items())
            .iter()
            .map(|item| item.name.clone())
            .collect()
    }

    #[test]
    fn slash_opens_session_commands_filtered_by_prefix() {
        assert_eq!(names(&typed("/")), ["/help", "/exit"]);
        assert_eq!(names(&typed("/he")), ["/help"]);
        assert!(names(&typed("/x")).is_empty());
        assert_eq!(names(&typed("co")), ["convert"]);
        assert!(names(&typed("convert x")).is_empty());
        assert!(names(&typed("")).is_empty());
    }

    #[test]
    fn enter_runs_the_selected_session_command() {
        let mut editor = typed("/");
        press(&mut editor, KeyCode::Down, &[]);
        assert_eq!(
            press(&mut editor, KeyCode::Enter, &[]),
            Action::Submit("/exit".into())
        );
    }

    #[test]
    fn choosing_a_cli_command_completes_it_with_a_space() {
        let mut editor = typed("co");
        assert_eq!(press(&mut editor, KeyCode::Enter, &[]), Action::Redraw);
        assert_eq!(editor.input, "convert ");
        assert!(names(&editor).is_empty());

        let mut editor = typed("con");
        press(&mut editor, KeyCode::Tab, &[]);
        assert_eq!(editor.input, "convert ");
    }

    #[test]
    fn enter_submits_the_trimmed_line_but_not_a_blank_one() {
        let mut editor = typed("  convert a.prisma  ");
        assert_eq!(
            press(&mut editor, KeyCode::Enter, &[]),
            Action::Submit("convert a.prisma".into())
        );
        assert_eq!(
            press(&mut typed("   "), KeyCode::Enter, &[]),
            Action::Redraw
        );
    }

    #[test]
    fn esc_closes_the_menu_until_the_next_edit() {
        let mut editor = typed("/");
        press(&mut editor, KeyCode::Esc, &[]);
        assert!(names(&editor).is_empty());
        press(&mut editor, KeyCode::Char('h'), &[]);
        assert_eq!(names(&editor), ["/help"]);
    }

    #[test]
    fn arrows_browse_history_and_restore_the_draft() {
        let history = vec!["first".to_owned(), "second".to_owned()];
        let mut editor = typed("draft");
        press(&mut editor, KeyCode::Up, &history);
        assert_eq!(editor.input, "second");
        press(&mut editor, KeyCode::Up, &history);
        press(&mut editor, KeyCode::Up, &history);
        assert_eq!(editor.input, "first");
        press(&mut editor, KeyCode::Down, &history);
        assert_eq!(editor.input, "second");
        press(&mut editor, KeyCode::Down, &history);
        assert_eq!(editor.input, "draft");
    }

    #[test]
    fn ctrl_c_clears_then_exits_when_pressed_twice() {
        let mut editor = typed("convert");
        assert_eq!(ctrl(&mut editor, 'c'), Action::Redraw);
        assert_eq!(editor.input, "");
        assert_eq!(ctrl(&mut editor, 'c'), Action::Redraw);
        assert!(editor.exit_armed);
        assert_eq!(ctrl(&mut editor, 'c'), Action::Exit);

        // Any other key disarms it.
        let mut editor = Editor::default();
        ctrl(&mut editor, 'c');
        press(&mut editor, KeyCode::Left, &[]);
        assert_eq!(ctrl(&mut editor, 'c'), Action::Redraw);
    }

    #[test]
    fn ctrl_d_exits_only_on_an_empty_line() {
        assert_eq!(ctrl(&mut Editor::default(), 'd'), Action::Exit);
        let mut editor = typed("ab");
        editor.cursor = 0;
        assert_eq!(ctrl(&mut editor, 'd'), Action::Redraw);
        assert_eq!(editor.input, "b");
    }

    #[test]
    fn editing_respects_multibyte_characters() {
        let mut editor = typed("café");
        press(&mut editor, KeyCode::Left, &[]);
        press(&mut editor, KeyCode::Backspace, &[]);
        assert_eq!(editor.input, "caé");
        press(&mut editor, KeyCode::Char('f'), &[]);
        press(&mut editor, KeyCode::End, &[]);
        press(&mut editor, KeyCode::Backspace, &[]);
        assert_eq!(editor.input, "caf");
    }

    #[test]
    fn ctrl_w_deletes_the_previous_word() {
        let mut editor = typed("convert a.prisma ");
        ctrl(&mut editor, 'w');
        assert_eq!(editor.input, "convert ");
    }
}
