use clap::Subcommand;
use transf_orm_core::config::GlobalConfig;
use transf_orm_core::registry;

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

    let orm = if let Some(name) = from {
        step(config, format!("Using ORM forced by --from: {name}"));
        registry::find_by_name(&name).unwrap_or_else(|| {
            eprintln!(
                "error: unknown ORM '{name}'. Valid options: {}",
                registry::names().join(", ")
            );
            std::process::exit(1);
        })
    } else {
        step(config, format!("Detecting ORM format for {path_str}"));
        let detected = registry::detect(&path_str).unwrap_or_else(|| {
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

        detected
    };
    step(config, format!("Source ORM: {}", orm.display_name));

    let make_importer = orm.make_importer.unwrap_or_else(|| {
        eprintln!("error: {} does not support importing", orm.display_name);
        std::process::exit(1);
    });

    step(config, format!("Reading {path_str}"));
    let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("error reading {path_str}: {e}");
        std::process::exit(1);
    });

    tracing::debug!(path = %path_str, bytes = content.len(), "read schema file");

    step(
        config,
        format!("Importing schema with the {} importer", orm.display_name),
    );
    match make_importer().import(&content) {
        Ok(schema) => {
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
            println!(
                "{}",
                schema
                    .to_canonical_json()
                    .expect("failed to serialize schema to JSON")
            );
        }

        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
