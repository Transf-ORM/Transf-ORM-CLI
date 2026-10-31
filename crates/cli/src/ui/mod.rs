//! Building blocks of the interactive interface, independent of the
//! Transf-ORM commands.
//!
//! # Adding an interactive box
//!
//! A command that needs more than text output (a list to pick from, a file
//! picker, a diff to browse…) returns a [`Panel`]: the interface draws it at
//! the bottom of the screen, in place of the input box, and sends it every
//! key until it closes.
//!
//! - For a list of choices, build a [`SelectList`]: sections, entries, and
//!   the [`PanelEvent`] each entry returns on Enter (run a line, fill the
//!   input box, or just close).
//! - For anything else, implement [`Panel`] on your own type: a title, the
//!   height wanted, how to draw and how to react to keys, plus a plain-text
//!   version for when there is no terminal.
//!
//! Then return it from the command as `Flow::Open(Box::new(panel))` (see
//! `repl::commands` for the `/help` example).

mod panel;
mod select_list;
pub(crate) mod theme;

pub(crate) use panel::{draw, height, Panel, PanelEvent};
pub(crate) use select_list::SelectList;
