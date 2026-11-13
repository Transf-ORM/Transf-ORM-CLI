use clap::Subcommand;
use transf_orm_core::api::{self, GlobalConfig};

/// Commands available in CLI mode.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Convert an ORM schema to the pivot IR (canonical JSON)
    Convert {
        /// Path to the schema file or directory
        path: std::path::PathBuf,

        /// Force the source ORM format (skips detection and confirmation)
        #[arg(long, value_name = "ORM")]
        from: Option<String>,
    },
}

/// Runs the CLI mode.
///
/// # Arguments
///
/// * `config` - Options from the global flags
/// * `command` - The subcommand to run, `None` for interactive mode
pub fn run(config: &GlobalConfig, command: Option<Command>) {
    match command {
        Some(Command::Convert { path, from }) => run_convert(config, path, from),
        None => println!("Interactive mode is not implemented yet (see #48)."),
    }
}

/// Prints a `--verbose` progress step on stderr, keeping stdout for the output.
fn step(config: &GlobalConfig, message: impl std::fmt::Display) {
    if config.verbose {
        eprintln!("[verbose] {message}");
    }
}

fn run_convert(config: &GlobalConfig, path: std::path::PathBuf, from: Option<String>) {
    let path_str = path.to_string_lossy();

    let (from, orm_display_name) = if let Some(name) = from {
        step(config, format!("Using ORM forced by --from: {name}"));

        let supported = api::orm_names();

        let canonical_name = supported
            .iter()
            .find(|candidate| candidate.eq_ignore_ascii_case(&name))
            .unwrap_or_else(|| {
                eprintln!(
                    "error: unknown ORM '{name}'. Valid options: {}",
                    supported.join(", ")
                );
                std::process::exit(1);
            });

        (canonical_name.to_string(), canonical_name.to_string())
    } else {
        step(config, format!("Detecting ORM format for {path_str}"));

        let detected = api::detect(&path).unwrap_or_else(|| {
            eprintln!("error: could not detect ORM format for '{path_str}'");
            eprintln!("Use --from to specify: transf-orm convert --from <orm> {path_str}");
            std::process::exit(1);
        });

        if !detected.certain {
            eprint!(
                "Detected: {} — press Enter to confirm, Ctrl+C to cancel: ",
                detected.display_name
            );

            let mut input = String::new();

            if std::io::stdin().read_line(&mut input).is_err() || !input.trim().is_empty() {
                eprintln!("Use --from to specify: transf-orm convert --from <orm> {path_str}");
                std::process::exit(1);
            }
        }

        (detected.name.to_owned(), detected.display_name.to_owned())
    };

    step(config, format!("Source ORM: {orm_display_name}"));
    step(config, format!("Reading {path_str}"));

    let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("error reading {path_str}: {e}");
        std::process::exit(1);
    });

    tracing::debug!(
        path = %path_str,
        bytes = content.len(),
        "read schema file"
    );

    step(
        config,
        format!("Importing schema with the {orm_display_name} importer"),
    );

    let schema = api::import(&content, &from).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    });

    step(
        config,
        format!(
            "Imported {} tables, {} views, {} enums",
            schema.tables.len(),
            schema.views.len(),
            schema.enums.len()
        ),
    );

    step(config, "Writing canonical JSON pivot to stdout");

    match api::export(&schema, "pivot") {
        Ok(output) => println!("{output}"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
