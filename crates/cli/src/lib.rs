use clap::Subcommand;
use transf_orm_core::config::GlobalConfig;
use transf_orm_core::registry;

mod repl;
mod ui;

/// Where commands write and ask questions: the real terminal in one-shot
/// mode, or the transcript of the interactive interface.
pub(crate) trait Console {
    /// Writes normal output (stdout in one-shot mode).
    fn out(&mut self, text: &str);
    /// Writes diagnostics: errors, `--verbose` steps (stderr in one-shot mode).
    fn err(&mut self, text: &str);
    /// Asks the user to confirm `question`; `true` when they do.
    fn confirm(&mut self, question: &str) -> bool;
}

/// [`Console`] on the process's stdout, stderr and stdin.
pub(crate) struct StdConsole;

impl Console for StdConsole {
    fn out(&mut self, text: &str) {
        println!("{text}");
    }

    fn err(&mut self, text: &str) {
        eprintln!("{text}");
    }

    fn confirm(&mut self, question: &str) -> bool {
        eprint!("{question} — press Enter to confirm, Ctrl+C to cancel: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).is_ok() && input.trim().is_empty()
    }
}

/// Commands available in CLI mode.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Convert an ORM schema to the pivot IR (canonical JSON)
    Convert {
        /// Path to the schema file; defaults to the schema detected in the
        /// current directory
        path: Option<std::path::PathBuf>,

        /// Force the source ORM format (skips detection and confirmation)
        #[arg(long, value_name = "ORM")]
        from: Option<String>,
    },
}

/// Runs the CLI mode.
///
/// Without a command, opens the interactive prompt in the current directory.
/// With one, runs it and exits with status 1 if it fails.
///
/// # Arguments
///
/// * `config` - Options from the global flags
/// * `command` - The subcommand to run, `None` for interactive mode
pub fn run(config: &GlobalConfig, command: Option<Command>) {
    match command {
        Some(command) => {
            if let Err(message) = execute(config, command, &mut StdConsole) {
                eprintln!("error: {message}");
                std::process::exit(1);
            }
        }
        None => repl::run(config),
    }
}

/// Runs one command, shared by the one-shot and the interactive modes.
///
/// # Errors
///
/// Returns the message to show the user when the command fails; the caller
/// decides whether to exit (one-shot) or keep the session open (interactive).
fn execute(
    config: &GlobalConfig,
    command: Command,
    console: &mut dyn Console,
) -> Result<(), String> {
    match command {
        Command::Convert { path, from } => run_convert(config, console, path, from),
    }
}

/// The schema file to use when a command is given no path: the only one
/// detected in the current directory.
///
/// # Errors
///
/// Returns a message asking for a path when no schema file, or more than
/// one, is found.
fn detected_schema() -> Result<std::path::PathBuf, String> {
    let cwd = std::env::current_dir()
        .map_err(|e| format!("could not read the current directory: {e}"))?;
    let mut detected = registry::detect_in_dir(&cwd);
    match detected.len() {
        0 => Err("no schema file found in the current directory; pass its path".to_owned()),
        1 => Ok(detected.remove(0).path),
        n => Err(format!(
            "{n} schema files found, pass the one to use:\n{}",
            detected
                .iter()
                .map(|schema| format!("  {}", schema.path.display()))
                .collect::<Vec<_>>()
                .join("\n")
        )),
    }
}

/// Writes a `--verbose` progress step as a diagnostic, keeping the normal
/// output clean.
fn step(config: &GlobalConfig, console: &mut dyn Console, message: impl std::fmt::Display) {
    if config.verbose {
        console.err(&format!("[verbose] {message}"));
    }
}

fn run_convert(
    config: &GlobalConfig,
    console: &mut dyn Console,
    path: Option<std::path::PathBuf>,
    from: Option<String>,
) -> Result<(), String> {
    let path = match path {
        Some(path) => path,
        None => {
            let path = detected_schema()?;
            console.err(&format!("Using detected schema: {}", path.display()));
            path
        }
    };
    let path_str = path.to_string_lossy();
    let use_from = format!("Use --from to specify: transf-orm convert --from <orm> {path_str}");

    let orm = if let Some(name) = from {
        step(
            config,
            console,
            format!("Using ORM forced by --from: {name}"),
        );
        registry::find_by_name(&name).ok_or_else(|| {
            format!(
                "unknown ORM '{name}'. Valid options: {}",
                registry::names().join(", ")
            )
        })?
    } else {
        step(
            config,
            console,
            format!("Detecting ORM format for {path_str}"),
        );
        let detected = registry::detect(&path_str)
            .ok_or_else(|| format!("could not detect ORM format for '{path_str}'\n{use_from}"))?;

        if !detected.certain && !console.confirm(&format!("Detected: {}", detected.display_name)) {
            return Err(format!("ORM detection not confirmed\n{use_from}"));
        }

        detected
    };
    step(config, console, format!("Source ORM: {}", orm.display_name));

    let make_importer = orm
        .make_importer
        .ok_or_else(|| format!("{} does not support importing", orm.display_name))?;

    step(config, console, format!("Reading {path_str}"));
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("could not read {path_str}: {e}"))?;

    tracing::debug!(path = %path_str, bytes = content.len(), "read schema file");

    step(
        config,
        console,
        format!("Importing schema with the {} importer", orm.display_name),
    );
    let schema = make_importer()
        .import(&content)
        .map_err(|e| e.to_string())?;
    step(
        config,
        console,
        format!(
            "Imported {} tables, {} views, {} enums",
            schema.tables.len(),
            schema.views.len(),
            schema.enums.len()
        ),
    );

    step(config, console, "Writing canonical JSON pivot to stdout");
    let json = schema
        .to_canonical_json()
        .map_err(|e| format!("could not serialize the pivot to JSON: {e}"))?;
    console.out(&json);
    Ok(())
}
