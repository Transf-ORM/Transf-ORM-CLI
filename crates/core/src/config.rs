use std::fmt;
use std::str::FromStr;

/// Log verbosity selected with the global `--debug <level>` flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl DebugLevel {
    /// Every accepted level, in increasing verbosity.
    pub const ALL: [DebugLevel; 5] = [
        DebugLevel::Error,
        DebugLevel::Warn,
        DebugLevel::Info,
        DebugLevel::Debug,
        DebugLevel::Trace,
    ];

    /// Lowercase name used on the command line.
    pub fn as_str(self) -> &'static str {
        match self {
            DebugLevel::Error => "error",
            DebugLevel::Warn => "warn",
            DebugLevel::Info => "info",
            DebugLevel::Debug => "debug",
            DebugLevel::Trace => "trace",
        }
    }
}

impl fmt::Display for DebugLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for DebugLevel {
    type Err = String;

    /// Parses a level name, case-insensitively.
    ///
    /// # Errors
    ///
    /// Returns a message listing the valid levels when `s` is not one of them.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "error" => Ok(DebugLevel::Error),
            "warn" => Ok(DebugLevel::Warn),
            "info" => Ok(DebugLevel::Info),
            "debug" => Ok(DebugLevel::Debug),
            "trace" => Ok(DebugLevel::Trace),
            _ => Err(format!(
                "invalid debug level '{s}' (expected one of: error, warn, info, debug, trace)"
            )),
        }
    }
}

/// Options set by the global flags of the `transf-orm` binary.
///
/// Shared by the CLI and server modes so both read the same settings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GlobalConfig {
    /// `--debug <level>`: log verbosity, `None` when not set.
    pub debug: Option<DebugLevel>,
    /// `--verbose`: print extra progress information.
    pub verbose: bool,
    /// `--perf`: report performance metrics.
    pub perf: bool,
    /// `--tts`: read CLI feedback aloud.
    pub tts: bool,
    /// `--daltonien`: use a colour-blind-friendly palette.
    pub daltonien: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_level_parses_every_level() {
        for level in DebugLevel::ALL {
            assert_eq!(level.as_str().parse::<DebugLevel>(), Ok(level));
        }
    }

    #[test]
    fn debug_level_is_case_insensitive() {
        assert_eq!("TRACE".parse::<DebugLevel>(), Ok(DebugLevel::Trace));
    }

    #[test]
    fn debug_level_rejects_unknown_value() {
        let err = "verbose".parse::<DebugLevel>().unwrap_err();
        assert!(err.contains("error, warn, info, debug, trace"));
    }
}
