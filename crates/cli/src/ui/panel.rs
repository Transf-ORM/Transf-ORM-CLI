//! The [`Panel`] trait: an interactive box that takes the place of the
//! input box at the bottom of the screen, and how it is framed.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Padding, Widget};

use super::theme::ACCENT;

/// What the session does after a key handled by a panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PanelEvent {
    /// Keep the panel open.
    Continue,
    /// Close the panel.
    Close,
    /// Close the panel and run this line, as if it had been typed.
    Run(String),
    /// Close the panel and put this text in the input box, to be completed.
    Fill(String),
}

/// An interactive box shown at the bottom of the screen while it is open.
///
/// While a panel is open it receives every key; it closes when
/// [`Panel::handle_key`] returns anything but [`PanelEvent::Continue`].
pub(crate) trait Panel {
    /// Shown in the top border.
    fn title(&self) -> &str;

    /// Rows of content wanted, without the border and the hint row. The
    /// panel may get fewer and must then scroll.
    fn height(&self) -> u16;

    /// Draws the content inside the border.
    fn render(&mut self, area: Rect, buf: &mut Buffer);

    /// Reacts to a key press.
    fn handle_key(&mut self, key: KeyEvent) -> PanelEvent;

    /// Keys reminder shown under the box.
    fn hint(&self) -> &str {
        "Esc to close"
    }

    /// The content as plain text, shown instead of the panel when there is
    /// no terminal to draw it in (pipes, scripts, tests).
    fn plain_text(&self) -> String;
}

/// Rows a panel takes on screen (border, content, hint row), at most `max`.
pub(crate) fn height(panel: &dyn Panel, max: u16) -> u16 {
    panel.height().saturating_add(3).min(max)
}

/// Draws `panel` in `area`: rounded accent border with the title, the
/// content inside, then the hint row below.
pub(crate) fn draw(panel: &mut dyn Panel, area: Rect, buf: &mut Buffer) {
    let [frame, hint] = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(ACCENT))
        .title(format!(" {} ", panel.title()))
        .padding(Padding::horizontal(1));
    let inner = block.inner(frame);
    block.render(frame, buf);
    panel.render(inner, buf);
    Line::styled(format!("  {}", panel.hint()), Style::new().dark_gray()).render(hint, buf);
}
