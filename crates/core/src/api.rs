//! Stable public façade for Transf-ORM.
//!
//! The CLI, server, and other consumers must use this module rather than
//! importing from the internal `pivot`, `importer`, `exporter`, or `registry`
//! modules. Additional operations (generate, diff, history) will be added by
//! their own issues.
//!
//! # Example
//!
//! ```
//! use transf_orm_core::api;
//!
//! let detected = api::detect("schema.prisma").unwrap();
//! assert_eq!(detected.name, "prisma");
//! ```

use std::path::Path;

use crate::registry;

pub use crate::config::{DebugLevel, GlobalConfig};
pub use crate::exporter::ExportError;
pub use crate::importer::ImportError;
pub use crate::pivot::metadata::UnknownFeature;
pub use crate::pivot::relation::{CascadeOptions, JunctionTable, Relation, RelationKind};
pub use crate::pivot::types::{DbHint, DefaultFn, LiteralValue};
pub use crate::pivot::{ColumnType, DatabaseKind, DefaultValue, ScalarType, Schema as Pivot};

/// An ORM identified from a schema file's extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DetectedOrm {
    /// Lowercase ORM identifier understood by the API.
    pub name: &'static str,
    /// Human-readable ORM name.
    pub display_name: &'static str,
    /// Whether detection requires no additional confirmation.
    pub certain: bool,
}

/// Result of a schema conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformOutput {
    /// Rendered schema in the requested target format.
    pub content: String,
    /// Source ORM identifier passed by the caller.
    pub from: String,
    /// Target ORM identifier passed by the caller.
    pub to: String,
}

/// Error returned by public façade operations.
#[derive(Debug)]
pub enum ApiError {
    /// The named source or target ORM is unknown.
    UnknownOrm(String),
    /// A registered ORM has no importer.
    ImportNotSupported(String),
    /// A registered ORM has no exporter.
    ExportNotSupported(String),
    /// The importer rejected an input schema.
    Import(ImportError),
    /// The exporter rejected a pivot schema.
    Export(ExportError),
    /// Pivot JSON serialization failed.
    Serialization(serde_json::Error),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownOrm(name) => write!(f, "unknown ORM '{name}'"),
            Self::ImportNotSupported(name) => write!(f, "{name} does not support importing"),
            Self::ExportNotSupported(name) => write!(f, "{name} does not support exporting"),
            Self::Import(error) => write!(f, "{error}"),
            Self::Export(error) => write!(f, "{error}"),
            Self::Serialization(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ApiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Import(error) => Some(error),
            Self::Export(error) => Some(error),
            Self::Serialization(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ImportError> for ApiError {
    fn from(error: ImportError) -> Self {
        Self::Import(error)
    }
}

impl From<ExportError> for ApiError {
    fn from(error: ExportError) -> Self {
        Self::Export(error)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}

/// Detect an ORM from a schema file path.
///
/// Returns `None` when the path has an unknown or missing extension.
///
/// # Examples
///
/// ```
/// use transf_orm_core::api::detect;
///
/// let detected = detect("schema.prisma").unwrap();
/// assert_eq!(detected.name, "prisma");
/// assert!(detect("schema.unknown").is_none());
/// ```
pub fn detect(path: impl AsRef<Path>) -> Option<DetectedOrm> {
    let path = path.as_ref().to_string_lossy();
    let orm = registry::detect(&path)?;

    Some(DetectedOrm {
        name: orm.name,
        display_name: orm.display_name,
        certain: orm.certain,
    })
}

/// Return lowercase names of all registered ORMs.
pub fn orm_names() -> Vec<&'static str> {
    registry::names()
}

/// Parse a source ORM schema into the public pivot representation.
///
/// # Errors
///
/// Returns [`ApiError::UnknownOrm`] if `from` is unknown,
/// [`ApiError::ImportNotSupported`] if no importer exists, or
/// [`ApiError::Import`] when parsing or validation fails.
///
/// # Examples
///
/// ```
/// use transf_orm_core::api;
///
/// let input = r#"
/// datasource db {
///   provider = "postgresql"
///   url = env("DATABASE_URL")
/// }
/// model User {
///   id Int @id
/// }
/// "#;
///
/// let pivot = api::import(input, "prisma").unwrap();
/// assert_eq!(pivot.tables.len(), 1);
/// ```
pub fn import(input: &str, from: &str) -> Result<Pivot, ApiError> {
    let orm = registry::find_by_name(from).ok_or_else(|| ApiError::UnknownOrm(from.to_owned()))?;

    let make_importer = orm
        .make_importer
        .ok_or_else(|| ApiError::ImportNotSupported(orm.display_name.to_owned()))?;

    Ok(make_importer().import(input)?)
}

/// Render a pivot as the specified target format.
///
/// The `pivot` and `json` targets both produce canonical pivot JSON.
///
/// # Errors
///
/// Returns [`ApiError::UnknownOrm`] for unknown targets,
/// [`ApiError::ExportNotSupported`] when no exporter exists, or an
/// exporter/serialization error when rendering fails.
///
/// # Examples
///
/// ```
/// use transf_orm_core::api::{self, DatabaseKind, Pivot};
///
/// let pivot = Pivot::new(DatabaseKind::PostgreSql);
/// let json = api::export(&pivot, "pivot").unwrap();
/// assert!(json.contains("pivotVersion"));
/// ```
pub fn export(pivot: &Pivot, to: &str) -> Result<String, ApiError> {
    if to.eq_ignore_ascii_case("pivot") || to.eq_ignore_ascii_case("json") {
        return Ok(pivot.to_canonical_json()?);
    }

    let orm = registry::find_by_name(to).ok_or_else(|| ApiError::UnknownOrm(to.to_owned()))?;

    let make_exporter = orm
        .make_exporter
        .ok_or_else(|| ApiError::ExportNotSupported(orm.display_name.to_owned()))?;

    Ok(make_exporter().export(pivot)?)
}

/// Convert a source ORM schema into a target representation.
///
/// This operation is equivalent to calling [`import`] followed by [`export`].
/// Currently, Prisma-to-pivot JSON is supported; target ORM exporters can be
/// registered independently.
///
/// # Errors
///
/// Propagates importer, exporter, registry, or serialization errors as [`ApiError`].
///
/// # Examples
///
/// ```
/// use transf_orm_core::api::transform;
///
/// let input = r#"
/// datasource db {
///   provider = "postgresql"
///   url = env("DATABASE_URL")
/// }
/// model User {
///   id Int @id
/// }
/// "#;
///
/// let output = transform(input, "prisma", "pivot").unwrap();
/// assert!(output.content.contains("pivotVersion"));
/// ```
pub fn transform(input: &str, from: &str, to: &str) -> Result<TransformOutput, ApiError> {
    let pivot = import(input, from)?;
    let content = export(&pivot, to)?;

    Ok(TransformOutput {
        content,
        from: from.to_owned(),
        to: to.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRISMA_SCHEMA: &str = r#"
        datasource db {
            provider = "postgresql"
            url = env("DATABASE_URL")
        }

        model User {
            id    Int    @id
            email String @unique
        }
    "#;

    #[test]
    fn detects_orm_through_facade() {
        let detected = detect("schema.prisma").expect("ORM should be detected");
        assert_eq!(detected.name, "prisma");
        assert_eq!(detected.display_name, "Prisma");
        assert!(detected.certain);
        assert!(detect("schema.other").is_none());
    }

    #[test]
    fn creates_and_serializes_pivot_through_facade() {
        let pivot = Pivot::new(DatabaseKind::PostgreSql);
        let content = export(&pivot, "pivot").expect("pivot export should succeed");
        let restored = Pivot::from_json(&content).expect("pivot JSON should parse");
        assert_eq!(pivot, restored);
    }

    #[test]
    fn imports_and_converts_through_facade_only() {
        let pivot = import(PRISMA_SCHEMA, "prisma").expect("schema should import");
        assert_eq!(pivot.tables.len(), 1);
        assert_eq!(pivot.tables[0].name, "User");

        let output = export(&pivot, "pivot").expect("pivot should serialize");
        assert!(output.contains("\"pivotVersion\""));
        assert!(output.contains("\"User\""));
    }

    #[test]
    fn transforms_through_facade() {
        let output =
            transform(PRISMA_SCHEMA, "prisma", "pivot").expect("transformation should succeed");
        assert_eq!(output.from, "prisma");
        assert_eq!(output.to, "pivot");
        assert!(output.content.contains("\"pivotVersion\""));
    }

    #[test]
    fn reports_unsupported_exports_without_panicking() {
        let pivot = Pivot::new(DatabaseKind::PostgreSql);
        assert!(matches!(
            export(&pivot, "prisma"),
            Err(ApiError::ExportNotSupported(_))
        ));
        assert!(matches!(
            import(PRISMA_SCHEMA, "unknown"),
            Err(ApiError::UnknownOrm(_))
        ));
    }
}
