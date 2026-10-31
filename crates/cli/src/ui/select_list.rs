//! [`SelectList`]: a ready-made [`Panel`] listing entries grouped in
//! sections, one of which is selected and chosen with Enter.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use unicode_width::UnicodeWidthStr;

use super::panel::{Panel, PanelEvent};
use super::theme::ACCENT;

enum Row {
    Section(String),
    Item {
        label: String,
        detail: String,
        on_enter: PanelEvent,
    },
}

/// A list of entries grouped in sections.
///
/// ↑/↓ move the selection (wrapping, skipping section headers), Home/End
/// jump to the first/last entry, Enter returns the selected entry's event,
/// Esc or Ctrl-C close the list.
///
/// ```text
/// let list = SelectList::new("Help")
///     .section("Session commands")
///     .item("/clear", "Clear the conversation", PanelEvent::Run("/clear".into()));
/// ```
pub(crate) struct SelectList {
    title: String,
    rows: Vec<Row>,
    /// Index in `rows` of the selected item, `None` while there is no item.
    selected: Option<usize>,
    /// First row shown, so the selection stays in view.
    offset: usize,
}

impl SelectList {
    pub(crate) fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            rows: Vec::new(),
            selected: None,
            offset: 0,
        }
    }

    /// Adds a section header; the items added next belong to it.
    pub(crate) fn section(mut self, title: impl Into<String>) -> Self {
        self.rows.push(Row::Section(title.into()));
        self
    }

    /// Adds an entry; `on_enter` is returned when it is chosen with Enter.
    pub(crate) fn item(
        mut self,
        label: impl Into<String>,
        detail: impl Into<String>,
        on_enter: PanelEvent,
    ) -> Self {
        self.rows.push(Row::Item {
            label: label.into(),
            detail: detail.into(),
            on_enter,
        });
        self.selected.get_or_insert(self.rows.len() - 1);
        self
    }

    fn items(&self) -> impl DoubleEndedIterator<Item = usize> + '_ {
        self.rows
            .iter()
            .enumerate()
            .filter(|(_, row)| matches!(row, Row::Item { .. }))
            .map(|(index, _)| index)
    }

    /// Moves the selection to the next (`forward`) or previous item,
    /// wrapping around.
    fn step(&mut self, forward: bool) {
        let Some(current) = self.selected else { return };
        let items: Vec<usize> = self.items().collect();
        let position = items.iter().position(|&i| i == current).unwrap_or(0);
        let next = if forward {
            (position + 1) % items.len()
        } else {
            (position + items.len() - 1) % items.len()
        };
        self.selected = Some(items[next]);
    }

    fn label_width(&self) -> usize {
        self.rows
            .iter()
            .filter_map(|row| match row {
                Row::Item { label, .. } => Some(label.width()),
                Row::Section(_) => None,
            })
            .max()
            .unwrap_or(0)
    }
}

impl Panel for SelectList {
    fn title(&self) -> &str {
        &self.title
    }

    fn height(&self) -> u16 {
        self.rows.len() as u16
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let view = usize::from(area.height);
        if let Some(selected) = self.selected {
            if selected < self.offset {
                // Show the section header right above the first item too.
                self.offset = selected.saturating_sub(1);
            } else if selected >= self.offset + view {
                self.offset = selected + 1 - view;
            }
        }

        let label_width = self.label_width();
        for (row_index, row) in self.rows.iter().enumerate().skip(self.offset).take(view) {
            let line = match row {
                Row::Section(title) => Line::styled(title.clone(), Style::new().dark_gray().bold()),
                Row::Item { label, detail, .. } => {
                    let label = format!("{label:<label_width$}");
                    if Some(row_index) == self.selected {
                        Line::from(vec![
                            Span::styled("› ", Style::new().fg(ACCENT)),
                            Span::styled(label, Style::new().fg(ACCENT).bold()),
                            Span::raw("   "),
                            Span::raw(detail.clone()),
                        ])
                    } else {
                        Line::from(vec![
                            Span::raw("  "),
                            Span::raw(label),
                            Span::raw("   "),
                            Span::styled(detail.clone(), Style::new().dark_gray()),
                        ])
                    }
                }
            };
            let y = area.y + (row_index - self.offset) as u16;
            line.render(
                Rect {
                    y,
                    height: 1,
                    ..area
                },
                buf,
            );
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> PanelEvent {
        match key.code {
            KeyCode::Esc => return PanelEvent::Close,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return PanelEvent::Close;
            }
            KeyCode::Up => self.step(false),
            KeyCode::Down => self.step(true),
            KeyCode::Home => {
                let first = self.items().next();
                self.selected = first;
            }
            KeyCode::End => {
                let last = self.items().next_back();
                self.selected = last;
            }
            KeyCode::Enter => {
                if let Some(Row::Item { on_enter, .. }) = self.selected.map(|i| &self.rows[i]) {
                    return on_enter.clone();
                }
            }
            _ => {}
        }
        PanelEvent::Continue
    }

    fn hint(&self) -> &str {
        "↑↓ navigate · Enter select · Esc close"
    }

    fn plain_text(&self) -> String {
        let label_width = self.label_width();
        let mut text = String::new();
        for row in &self.rows {
            match row {
                Row::Section(title) => {
                    if !text.is_empty() {
                        text.push('\n');
                    }
                    text.push_str(&format!("{title}:\n"));
                }
                Row::Item { label, detail, .. } => {
                    text.push_str(&format!("  {label:<label_width$}   {detail}\n"));
                }
            }
        }
        text.trim_end().to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn list() -> SelectList {
        SelectList::new("Help")
            .section("First")
            .item("a", "about a", PanelEvent::Run("a".into()))
            .item("b", "about b", PanelEvent::Fill("b ".into()))
            .section("Second")
            .item("c", "about c", PanelEvent::Close)
    }

    fn press(list: &mut SelectList, code: KeyCode) -> PanelEvent {
        list.handle_key(KeyEvent::from(code))
    }

    #[test]
    fn arrows_skip_sections_and_wrap() {
        let mut list = list();
        assert_eq!(
            press(&mut list, KeyCode::Enter),
            PanelEvent::Run("a".into())
        );
        press(&mut list, KeyCode::Down);
        assert_eq!(
            press(&mut list, KeyCode::Enter),
            PanelEvent::Fill("b ".into())
        );
        press(&mut list, KeyCode::Down);
        assert_eq!(press(&mut list, KeyCode::Enter), PanelEvent::Close);
        press(&mut list, KeyCode::Down);
        assert_eq!(
            press(&mut list, KeyCode::Enter),
            PanelEvent::Run("a".into())
        );
        press(&mut list, KeyCode::Up);
        assert_eq!(press(&mut list, KeyCode::Enter), PanelEvent::Close);
    }

    #[test]
    fn home_end_and_escape() {
        let mut list = list();
        press(&mut list, KeyCode::End);
        assert_eq!(press(&mut list, KeyCode::Enter), PanelEvent::Close);
        press(&mut list, KeyCode::Home);
        assert_eq!(
            press(&mut list, KeyCode::Enter),
            PanelEvent::Run("a".into())
        );
        assert_eq!(press(&mut list, KeyCode::Esc), PanelEvent::Close);
        assert_eq!(press(&mut list, KeyCode::Char('x')), PanelEvent::Continue);
    }

    #[test]
    fn empty_list_does_nothing() {
        let mut list = SelectList::new("Empty");
        press(&mut list, KeyCode::Down);
        assert_eq!(press(&mut list, KeyCode::Enter), PanelEvent::Continue);
    }

    #[test]
    fn selection_stays_in_view_when_scrolling() {
        let mut list = list();
        let mut terminal = Terminal::new(TestBackend::new(30, 2)).unwrap();
        press(&mut list, KeyCode::End);
        terminal
            .draw(|frame| list.render(frame.area(), frame.buffer_mut()))
            .unwrap();
        let buffer = terminal.backend().buffer();
        let rows: Vec<String> = (0..2)
            .map(|y| (0..30).map(|x| buffer[(x, y)].symbol()).collect())
            .collect();
        assert!(rows[0].starts_with("Second"), "{rows:?}");
        assert!(rows[1].starts_with("› c"), "{rows:?}");
    }

    #[test]
    fn plain_text_lists_sections_and_aligned_items() {
        assert_eq!(
            list().plain_text(),
            "First:\n  a   about a\n  b   about b\n\nSecond:\n  c   about c"
        );
    }
}
