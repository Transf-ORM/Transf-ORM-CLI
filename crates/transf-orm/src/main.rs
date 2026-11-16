use std::io::IsTerminal;

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};
use transf_orm_cli::Command;
use transf_orm_core::config::{DebugLevel, GlobalConfig};
use transf_orm_server::ServerArgs;

#[derive(Parser)]
#[command(name = "transf-orm", version, about = "ORM schema converter")]
struct Args {
    /// Run the interactive CLI (default mode)
    #[arg(long, conflicts_with = "serveur")]
    cli: bool,

    /// Run the HTTP server
    #[arg(long)]
    serveur: bool,

    /// Log level: error, warn, info, debug or trace
    #[arg(long, value_name = "LEVEL", global = true)]
    debug: Option<DebugLevel>,

    /// Print extra progress information
    #[arg(long, global = true)]
    verbose: bool,

    /// Report performance metrics
    #[arg(long, global = true)]
    perf: bool,

    /// Read CLI feedback aloud
    #[arg(long, global = true)]
    tts: bool,

    /// Use a colour-blind-friendly palette
    #[arg(long, global = true)]
    daltonien: bool,

    #[command(flatten)]
    server: ServerArgs,

    #[command(subcommand)]
    command: Option<Command>,
}

fn main() {
    dispatch(Args::parse());
}

/// Routes to the server mode with `--serveur`, to the CLI mode otherwise.
fn dispatch(args: Args) {
    let config = GlobalConfig {
        debug: args.debug,
        verbose: args.verbose,
        perf: args.perf,
        tts: args.tts,
        daltonien: args.daltonien,
    };

    if let Some(level) = config.debug {
        init_logging(level);
    }

    if args.serveur {
        if args.command.is_some() {
            Args::command()
                .error(
                    ErrorKind::ArgumentConflict,
                    "--serveur cannot be used with a CLI command",
                )
                .exit();
        }
        if let Err(err) = transf_orm_server::run(&config, args.server) {
            eprintln!("error: could not start the server: {err}");
            std::process::exit(1);
        }
    } else {
        transf_orm_cli::run(&config, args.command);
    }
}

/// Sends the internal `tracing` logs to stderr, up to `level`.
///
/// Only called with `--debug`: without it no subscriber is installed and the
/// output stays unchanged.
fn init_logging(level: DebugLevel) {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(level)
        .with_ansi(std::io::stderr().is_terminal())
        .without_time()
        .init();
}
