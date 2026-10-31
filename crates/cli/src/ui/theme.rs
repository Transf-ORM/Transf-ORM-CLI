//! Colors shared by every part of the interface.

use ratatui::style::Color;

/// Borders of focused boxes, the prompt and selected entries.
pub(crate) const ACCENT: Color = Color::Cyan;
/// Error messages.
pub(crate) const ERROR: Color = Color::LightRed;
