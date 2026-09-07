pub mod prisma;

use crate::pivot::Schema;

/// Contract that every ORM importer must satisfy.
pub trait Importer {
    /// Parse an ORM schema string and return the pivot [`Schema`].
    ///
    /// # Errors
    ///
    /// Returns [`ImportError`] if the input cannot be parsed or is structurally invalid.
    fn import(&self, input: &str) -> Result<Schema, ImportError>;
}

/// Error returned by any importer when parsing or resolving a schema fails.
#[derive(Debug)]
pub enum ImportError {
    /// The source file could not be parsed — contains a human-readable description.
    Parse(String),
    /// The parsed structure is semantically invalid (e.g. unknown type reference).
    Schema(String),
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::Parse(s) => write!(f, "parse error: {}", s),
            ImportError::Schema(s) => write!(f, "schema error: {}", s),
        }
    }
}

impl std::error::Error for ImportError {}
