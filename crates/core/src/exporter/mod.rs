use crate::pivot::Schema;

/// Contract that every ORM exporter must satisfy.
pub trait Exporter {
    /// Serialize a pivot [`Schema`] into the target ORM's schema format.
    ///
    /// # Errors
    ///
    /// Returns [`ExportError`] if the schema cannot be represented in the target format.
    fn export(&self, schema: &Schema) -> Result<String, ExportError>;
}

/// Error returned by any exporter when serializing a schema fails.
#[derive(Debug)]
pub enum ExportError {
    /// The pivot schema contains a construct that has no equivalent in the target ORM.
    Unsupported(String),
    /// Serialization failed for an internal reason.
    Render(String),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::Unsupported(s) => write!(f, "unsupported construct: {}", s),
            ExportError::Render(s) => write!(f, "render error: {}", s),
        }
    }
}

impl std::error::Error for ExportError {}
