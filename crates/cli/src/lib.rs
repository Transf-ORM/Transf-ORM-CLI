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
    let _ = config; // consumed by the --verbose / --debug / --perf / --tts / --daltonien features

    match command {
        Some(Command::Convert { path, from }) => run_convert(path, from),
        None => println!("Interactive mode is not implemented yet (see #48)."),
    }
}

fn run_convert(path: std::path::PathBuf, from: Option<String>) {
    let path_str = path.to_string_lossy();

    let orm = if let Some(name) = from {
        registry::find_by_name(&name).unwrap_or_else(|| {
            eprintln!(
                "error: unknown ORM '{name}'. Valid options: {}",
                registry::names().join(", ")
            );
            std::process::exit(1);
        })
    } else {
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

    let make_importer = orm.make_importer.unwrap_or_else(|| {
        eprintln!("error: {} does not support importing", orm.display_name);
        std::process::exit(1);
    });

    let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("error reading {path_str}: {e}");
        std::process::exit(1);
    });

    match make_importer().import(&content) {
        Ok(schema) => println!(
            "{}",
            schema
                .to_canonical_json()
                .expect("failed to serialize schema to JSON")
        ),

        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
