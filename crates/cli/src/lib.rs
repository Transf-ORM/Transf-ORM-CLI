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
    let _ = config; // consumed by the --verbose / --debug / --perf / --tts / --daltonien features

    match command {
        Some(Command::Convert { path, from }) => run_convert(path, from),
        None => println!("Interactive mode is not implemented yet (see #48)."),
    }
}

fn run_convert(path: std::path::PathBuf, from: Option<String>) {
    let path_str = path.to_string_lossy();

    let from = if let Some(name) = from {
        let supported = api::orm_names();
        if !supported
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(&name))
        {
            eprintln!(
                "error: unknown ORM '{name}'. Valid options: {}",
                supported.join(", ")
            );
            std::process::exit(1);
        }

        name
    } else {
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

        detected.name.to_owned()
    };

    let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("error reading {path_str}: {e}");
        std::process::exit(1);
    });

    match api::transform(&content, &from, "pivot") {
        Ok(output) => println!("{}", output.content),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
