use transf_orm_core::config::GlobalConfig;

/// Flags specific to the server mode.
#[derive(clap::Args, Debug)]
pub struct ServerArgs {
    /// Port to listen on in server mode (0 = any free port)
    #[arg(long, default_value_t = 3000, requires = "serveur")]
    pub port: u16,
}

/// Runs the server mode.
///
/// Only logs the startup for now; the HTTP bootstrap comes with #91.
///
/// # Arguments
///
/// * `config` - Options from the global flags
/// * `args` - Server-specific flags
pub fn run(config: &GlobalConfig, args: ServerArgs) {
    let _ = config; // consumed by the --verbose / --debug / --perf features

    eprintln!("Starting transf-orm server on port {}", args.port);
}
