mod error;
pub mod prisma;

pub use error::{ImportError, ImportErrorKind, SourceLocation};

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
