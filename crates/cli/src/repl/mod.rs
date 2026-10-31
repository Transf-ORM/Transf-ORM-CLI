//! Interactive mode: a prompt loop in the current directory.
//!
//! Lines starting with `/` are session commands (`/help`, `/exit`); any other
//! line is parsed as a CLI command, exactly as typed after `transf-orm` in a
//! shell (e.g. `convert schema.prisma`).

use std::io::{BufRead, IsTerminal};
use std::path::PathBuf;

use clap::{CommandFactory, Parser};
use transf_orm_core::config::GlobalConfig;
use transf_orm_core::registry::{self, DetectedSchema};

use crate::ui::Panel;
use crate::{execute, Command, Console, StdConsole};

mod commands;
mod editor;
mod tui;

use commands::SESSION_COMMANDS;
use editor::MenuItem;

/// A line typed at the prompt that is not a session command.
#[derive(Parser)]
#[command(no_binary_name = true, disable_help_subcommand = true)]
struct Line {
    #[command(subcommand)]
    command: Command,
}

/// What the session does after a line.
enum Flow {
    Continue,
    /// Clear the conversation shown on screen.
    Clear,
    /// Open an interactive box (see [`crate::ui`]).
    Open(Box<dyn Panel>),
    Exit,
}

/// What the session knows about the current directory when it starts.
struct Startup {
    cwd: PathBuf,
    detected: Vec<DetectedSchema>,
}

impl Startup {
    fn detect() -> Self {
        let cwd = std::env::current_dir().unwrap_or_default();
        let detected = registry::detect_in_dir(&cwd);
        Self { cwd, detected }
    }

    /// `"<file>"`, plus how many other schema files were found.
    fn files_summary(first: &DetectedSchema, others: usize) -> String {
        let path = first.path.display();
        match others {
            0 => format!("{path}"),
            1 => format!("{path} and 1 other schema file"),
            n => format!("{path} and {n} other schema files"),
        }
    }
}

/// Runs the interactive prompt until `/exit`, Ctrl-D or end of input.
///
/// In a terminal, the session is a full-screen interface on the alternate
/// screen, restored on exit. Otherwise (pipe, script, tests) lines are read
/// one by one from stdin. Either way a failing command prints its error and
/// the session goes on.
///
/// # Arguments
///
/// * `config` - Options from the global flags, applied to every command
pub(crate) fn run(config: &GlobalConfig) {
    let startup = Startup::detect();
    if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
        if let Err(err) = tui::run(config, &startup, menu_items()) {
            eprintln!("error: interactive interface failed: {err}");
        }
    } else {
        run_lines(config, &startup);
    }
}

/// Reads commands from a non-terminal stdin, without prompt or editing.
fn run_lines(config: &GlobalConfig, startup: &Startup) {
    println!(
        "Transf-ORM {} — interactive mode",
        env!("CARGO_PKG_VERSION")
    );
    match startup.detected.first() {
        Some(first) => println!(
            "Detected ORM: {} ({})",
            first.orm.display_name,
            Startup::files_summary(first, startup.detected.len() - 1)
        ),
        None => println!("No ORM detected: use --from to choose the source ORM"),
    }
    println!("Type /help for the list of commands, /exit to quit.");

    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match handle_line(config, line, &mut StdConsole) {
            Flow::Exit => break,
            // No terminal to draw the box in: show its content as text.
            Flow::Open(panel) => println!("{}", panel.plain_text()),
            Flow::Continue | Flow::Clear => {}
        }
    }
}

/// Entries of the prompt's command menu: session commands, then CLI
/// commands from their clap definitions.
fn menu_items() -> Vec<MenuItem> {
    let session = SESSION_COMMANDS.iter().map(|command| MenuItem {
        name: command.name.to_owned(),
        about: command.about.to_owned(),
        takes_args: false,
    });
    let commands = Line::command()
        .get_subcommands()
        .map(|command| MenuItem {
            name: command.get_name().to_owned(),
            about: command
                .get_about()
                .map(ToString::to_string)
                .unwrap_or_default(),
            takes_args: true,
        })
        .collect::<Vec<_>>();
    session.chain(commands).collect()
}

/// Runs one non-empty line typed at the prompt, writing to `console`.
fn handle_line(config: &GlobalConfig, line: &str, console: &mut dyn Console) -> Flow {
    if line.starts_with('/') {
        return match commands::find(line) {
            Some(command) => (command.run)(),
            None => {
                console.err(&format!(
                    "error: unknown command '{line}', type /help for the list"
                ));
                Flow::Continue
            }
        };
    }

    let words = match shell_words::split(line) {
        Ok(words) => words,
        Err(err) => {
            console.err(&format!("error: {err}"));
            return Flow::Continue;
        }
    };
    // clap's own message for an unknown command would show a meaningless
    // usage line, so it is handled here.
    if Line::command().find_subcommand(&words[0]).is_none() {
        console.err(&format!(
            "error: unknown command '{}', type /help for the list",
            words[0]
        ));
        return Flow::Continue;
    }

    let name = words[0].clone();
    match Line::try_parse_from(words) {
        // Also covers `convert --help`, which clap reports as an "error".
        Err(err) => {
            let text = err.render().to_string();
            if err.use_stderr() {
                // A bare `--help` means nothing at the prompt: point to the
                // command's help and to /help instead.
                let text = text.replace("try '--help'", &format!("try '{name} --help' or /help"));
                console.err(text.trim_end());
            } else {
                console.out(text.trim_end());
            }
        }
        Ok(Line { command }) => {
            if let Err(message) = execute(config, command, console) {
                console.err(&format!("error: {message}"));
            }
        }
    }
    Flow::Continue
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> GlobalConfig {
        GlobalConfig {
            debug: None,
            verbose: false,
            perf: false,
            tts: false,
            daltonien: false,
        }
    }

    fn flow(line: &str) -> Flow {
        handle_line(&config(), line, &mut StdConsole)
    }

    #[test]
    fn session_commands_return_their_flow() {
        assert!(matches!(flow("/exit"), Flow::Exit));
        assert!(matches!(flow("/clear"), Flow::Clear));
        assert!(matches!(flow("/help"), Flow::Open(_)));
    }

    #[test]
    fn other_lines_keep_the_session_open() {
        for line in [
            "/nope",
            "nope",
            "convert",
            "convert --help",
            "convert 'unterminated",
            "convert /nonexistent/schema.prisma",
        ] {
            assert!(matches!(flow(line), Flow::Continue), "{line}");
        }
    }

    /// [`Console`] that keeps what is written, for assertions.
    #[derive(Default)]
    struct Recorder {
        err: String,
    }

    impl Console for Recorder {
        fn out(&mut self, _: &str) {}

        fn err(&mut self, text: &str) {
            self.err.push_str(text);
        }

        fn confirm(&mut self, _: &str) -> bool {
            false
        }
    }

    #[test]
    fn argument_errors_point_to_the_command_help() {
        let mut recorder = Recorder::default();
        handle_line(&config(), "convert --nope", &mut recorder);
        assert!(
            recorder.err.contains("try 'convert --help' or /help"),
            "{}",
            recorder.err
        );
        assert!(!recorder.err.contains("try '--help'"));
    }

    #[test]
    fn lines_parse_like_shell_arguments() {
        let words = shell_words::split("convert 'my schema.prisma' --from prisma").unwrap();
        let Line { command } = Line::try_parse_from(words).unwrap();
        let Command::Convert { path, from } = command;
        assert_eq!(path, Some(std::path::PathBuf::from("my schema.prisma")));
        assert_eq!(from.as_deref(), Some("prisma"));
    }
}
