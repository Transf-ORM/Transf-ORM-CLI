//! Session commands: the `/name` commands of the interactive mode.
//!
//! Adding one is an entry in [`SESSION_COMMANDS`] and the function it runs;
//! the `/` menu and the `/help` box are generated from this table.

use clap::CommandFactory;

use super::{Flow, Line};
use crate::ui::{PanelEvent, SelectList};

/// A `/name` command of the interactive mode.
pub(super) struct SessionCommand {
    /// Typed name, with its leading `/`.
    pub name: &'static str,
    /// One-line description for the `/` menu and `/help`.
    pub about: &'static str,
    /// What the command does.
    pub run: fn() -> Flow,
}

pub(super) const SESSION_COMMANDS: &[SessionCommand] = &[
    SessionCommand {
        name: "/help",
        about: "Show the commands and what they do",
        run: || Flow::Open(Box::new(help_panel())),
    },
    SessionCommand {
        name: "/clear",
        about: "Clear the conversation",
        run: || Flow::Clear,
    },
    SessionCommand {
        name: "/exit",
        about: "Quit the session (or Ctrl-D)",
        run: || Flow::Exit,
    },
];

/// Finds a session command by its typed name (with the `/`).
pub(super) fn find(name: &str) -> Option<&'static SessionCommand> {
    SESSION_COMMANDS.iter().find(|command| command.name == name)
}

/// The `/help` box: the session commands (Enter runs them), then the CLI
/// commands from their clap definitions (Enter puts them in the input box,
/// since they need arguments).
fn help_panel() -> SelectList {
    let mut help = SelectList::new("Help").section("Session commands");
    for command in SESSION_COMMANDS {
        help = help.item(
            command.name,
            command.about,
            PanelEvent::Run(command.name.to_owned()),
        );
    }

    help = help.section("Commands (add --help for their options)");
    for command in Line::command().get_subcommands() {
        let about = command
            .get_about()
            .map(ToString::to_string)
            .unwrap_or_default();
        help = help.item(
            command.get_name(),
            about,
            PanelEvent::Fill(format!("{} ", command.get_name())),
        );
    }
    help
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::Panel;
    use ratatui::crossterm::event::{KeyCode, KeyEvent};

    #[test]
    fn help_lists_session_and_cli_commands() {
        let text = help_panel().plain_text();
        for command in SESSION_COMMANDS {
            assert!(text.contains(command.name), "{text}");
        }
        assert!(text.contains("convert"), "{text}");
    }

    #[test]
    fn enter_runs_session_commands_and_fills_cli_commands() {
        let mut help = help_panel();
        let enter = |help: &mut SelectList| help.handle_key(KeyEvent::from(KeyCode::Enter));
        assert_eq!(enter(&mut help), PanelEvent::Run("/help".into()));
        help.handle_key(KeyEvent::from(KeyCode::End));
        assert_eq!(enter(&mut help), PanelEvent::Fill("convert ".into()));
    }

    #[test]
    fn find_matches_the_typed_name() {
        assert!(find("/clear").is_some());
        assert!(find("clear").is_none());
    }
}
