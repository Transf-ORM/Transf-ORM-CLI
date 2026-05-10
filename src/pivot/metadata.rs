use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// ORM framework identifier — used to tag the source of a schema and to key target hints.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OrmKind {
    Prisma,
    Drizzle,
    TypeOrm,
    Sequelize,
    MikroOrm,
    Gorm,
    SqlAlchemy,
    ActiveRecord,
    Hibernate,
    EntityFramework,
    Custom(String),
}

/// An ORM-specific feature that has no equivalent in the target ORM.
///
/// Rather than silently dropping the feature or aborting the conversion, it is
/// stored here so that:
/// - the round-trip back to the source ORM can restore it (`recoverable = true`)
/// - exporters can emit a warning instead of silently losing information
///
/// # Examples
///
/// Prisma's `@ignore` attribute has no Drizzle equivalent:
///
/// ```
/// use transf_orm_cli::pivot::metadata::UnknownFeature;
///
/// let feature = UnknownFeature {
///     name: "prisma.ignore".to_string(),
///     value: serde_json::Value::Bool(true),
///     recoverable: true,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnknownFeature {
    /// Namespaced key identifying the feature, e.g. `"prisma.ignore"`, `"typeorm.version"`.
    pub name: String,
    pub value: Value,
    /// `true` if the feature can be restored when converting back to the source ORM.
    pub recoverable: bool,
}

/// ORM-specific metadata attached to tables, columns, and other IR nodes.
///
/// Acts as a universal escape hatch: features that cannot be expressed in the IR
/// are stored here rather than being silently dropped. This keeps conversions
/// lossless even when no semantic equivalent exists in the target ORM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OrmMetadata {
    /// The ORM this node was originally parsed from.
    pub source_orm: Option<OrmKind>,
    /// Hints for specific target ORMs, keyed by the serialized [`OrmKind`] string.
    ///
    /// Allows importers to embed target-specific generation hints without
    /// breaking the source-agnostic nature of the IR.
    pub target_hints: BTreeMap<String, Value>,
    /// Features from the source ORM with no IR equivalent.
    pub unknown_features: Vec<UnknownFeature>,
}
