/// Pivot IR — the intermediate representation at the heart of Transf-ORM.
///
/// Every importer parses a source ORM schema and produces a [`Schema`].
/// Every exporter consumes a [`Schema`] and emits a target ORM schema.
/// This crate-level module re-exports the most commonly used types for convenience.
///
/// # Architecture
///
/// The IR has two distinct layers stacked on each other:
///
/// - **DB layer** — everything that exists in the database DDL: [`Table`], [`View`],
///   [`MaterializedView`], [`Enum`], [`Sequence`], [`Function`], [`StoredProcedure`],
///   [`Trigger`], [`table::ForeignKey`](relation::ForeignKey), indexes, constraints.
///
/// - **ORM layer** — abstractions the ORM adds: [`Relation`], [`table::Behavior`],
///   [`table::Inheritance`], [`OrmMetadata`].
///
/// # Serialization
///
/// [`Schema::to_canonical_json`] produces deterministic output (keys sorted
/// alphabetically) so that `git diff` on stored pivot files is meaningful.
pub mod metadata;
pub mod procedure;
pub mod relation;
pub mod table;
pub mod types;
pub mod view;

pub use metadata::{OrmKind, OrmMetadata, UnknownFeature};
pub use procedure::{Function, StoredProcedure, Trigger};
pub use relation::{ForeignKey, Relation};
pub use table::{Column, Index, Table};
pub use types::{ColumnType, DefaultValue, ScalarType};
pub use view::{MaterializedView, View};

use serde::{Deserialize, Serialize};

/// Target database engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DatabaseKind {
    PostgreSql,
    MySql,
    MariaDb,
    Sqlite,
    MsSql,
    Oracle,
    CockroachDb,
    Custom(String),
}

/// A named schema within a database (PostgreSQL `schema`, MySQL `database`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbSchema {
    pub name: String,
}

/// A database extension (e.g. PostgreSQL `uuid-ossp`, `postgis`, `pgcrypto`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Extension {
    pub name: String,
    pub version: Option<String>,
    /// Schema in which the extension is installed.
    pub schema: Option<String>,
}

/// A single value in a database [`Enum`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumValue {
    /// Logical name exposed by the ORM.
    pub name: String,
    /// Actual value stored in the database when it differs from `name`.
    pub db_name: Option<String>,
}

/// A database enum type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Enum {
    pub name: String,
    pub db_name: Option<String>,
    pub db_schema: Option<String>,
    pub values: Vec<EnumValue>,
}

/// A field inside a PostgreSQL composite type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositeTypeField {
    pub name: String,
    pub field_type: types::ColumnType,
}

/// A PostgreSQL composite type — a named record type composed of typed fields.
///
/// Composite types can be used as column types, function return types, or
/// function parameter types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositeType {
    pub name: String,
    pub db_schema: Option<String>,
    pub fields: Vec<CompositeTypeField>,
}

/// A database sequence — an auto-incrementing integer generator.
///
/// Used as the backing mechanism for `SERIAL` and `BIGSERIAL` columns in PostgreSQL,
/// or referenced explicitly via `NEXTVAL`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sequence {
    pub name: String,
    pub db_schema: Option<String>,
    pub start: i64,
    pub increment: i64,
    pub min_value: Option<i64>,
    pub max_value: Option<i64>,
    /// Whether the sequence wraps around to `min_value` after reaching `max_value`.
    pub cycle: bool,
}

/// Root type of the pivot IR — the single source of truth for a database schema.
///
/// All importers produce a `Schema`; all exporters consume one. The two-layer
/// design (DB layer + ORM layer) means a single `Schema` carries enough information
/// to round-trip between any two supported ORMs without information loss.
///
/// # Versioning
///
/// `pivot_version` tracks the IR format version independently of the CLI version.
/// Increment it on breaking changes and ship a migration so existing serialized
/// schemas remain readable.
///
/// # Serialization
///
/// `serde_json` uses `BTreeMap` by default (without the `preserve_order` feature),
/// so JSON object keys are already sorted alphabetically. [`Schema::to_canonical_json`]
/// relies on this property to produce deterministic output suitable for `git diff`.
///
/// # Examples
///
/// ```
/// use transf_orm_cli::pivot::{DatabaseKind, Schema};
///
/// let schema = Schema::new(DatabaseKind::PostgreSql);
/// let json = schema.to_canonical_json().unwrap();
/// let restored = Schema::from_json(&json).unwrap();
/// assert_eq!(schema, restored);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    /// Semantic version of the pivot IR format itself.
    ///
    /// Must be incremented on breaking changes so stored schemas can be migrated.
    pub pivot_version: String,
    pub database: DatabaseKind,
    /// Optional version string for the target database engine (e.g. `"15.3"`).
    pub dialect_version: Option<String>,
    /// Named schemas (PostgreSQL) or databases (MySQL) within the instance.
    pub db_schemas: Vec<DbSchema>,
    /// Database extensions required by this schema.
    pub extensions: Vec<Extension>,
    pub tables: Vec<table::Table>,
    pub views: Vec<view::View>,
    pub materialized_views: Vec<view::MaterializedView>,
    pub enums: Vec<Enum>,
    pub composite_types: Vec<CompositeType>,
    pub sequences: Vec<Sequence>,
    pub functions: Vec<procedure::Function>,
    pub procedures: Vec<procedure::StoredProcedure>,
    pub triggers: Vec<procedure::Trigger>,
}

impl Schema {
    /// Create an empty schema for the given database with `pivot_version = "1.0.0"`.
    pub fn new(database: DatabaseKind) -> Self {
        Self {
            pivot_version: "1.0.0".to_string(),
            database,
            dialect_version: None,
            db_schemas: Vec::new(),
            extensions: Vec::new(),
            tables: Vec::new(),
            views: Vec::new(),
            materialized_views: Vec::new(),
            enums: Vec::new(),
            composite_types: Vec::new(),
            sequences: Vec::new(),
            functions: Vec::new(),
            procedures: Vec::new(),
            triggers: Vec::new(),
        }
    }

    /// Serialize to canonical JSON — pretty-printed with alphabetically sorted keys.
    ///
    /// # Errors
    ///
    /// Returns an error if any field cannot be serialized (e.g. a non-finite float
    /// inside a [`types::LiteralValue::Float`] default value).
    pub fn to_canonical_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize a `Schema` from a canonical JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON is malformed or does not match the current
    /// IR structure. Older pivot versions may require a migration before parsing.
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::metadata::OrmMetadata;
    use super::relation::{CascadeOptions, JunctionTable, RelationKind};
    use super::table::{Behavior, Column, PrimaryKey, Table};
    use super::types::{ColumnType, DefaultFn, DefaultValue, ScalarType};

    fn user_table() -> Table {
        Table {
            name: "User".to_string(),
            db_name: Some("users".to_string()),
            db_schema: None,
            columns: vec![
                Column {
                    name: "id".to_string(),
                    db_name: None,
                    col_type: ColumnType::simple(ScalarType::Int),
                    nullable: false,
                    default: Some(DefaultValue::Function(DefaultFn::AutoIncrement)),
                },
                Column {
                    name: "email".to_string(),
                    db_name: None,
                    col_type: ColumnType::simple(ScalarType::String),
                    nullable: false,
                    default: None,
                },
                Column {
                    name: "createdAt".to_string(),
                    db_name: Some("created_at".to_string()),
                    col_type: ColumnType::simple(ScalarType::Timestamp {
                        precision: 3,
                        with_timezone: true,
                    }),
                    nullable: false,
                    default: Some(DefaultValue::Function(DefaultFn::Now)),
                },
            ],
            primary_key: Some(PrimaryKey { name: None, columns: vec!["id".to_string()] }),
            indexes: Vec::new(),
            unique_constraints: Vec::new(),
            check_constraints: Vec::new(),
            foreign_keys: Vec::new(),
            relations: Vec::new(),
            inheritance: None,
            behaviors: vec![Behavior::CreatedAt { column: "createdAt".to_string() }],
            metadata: OrmMetadata::default(),
        }
    }

    #[test]
    fn schema_round_trips_through_json() {
        let mut schema = Schema::new(DatabaseKind::PostgreSql);
        schema.tables.push(user_table());

        let json = schema.to_canonical_json().expect("serialization failed");
        let restored = Schema::from_json(&json).expect("deserialization failed");

        assert_eq!(schema, restored);
    }

    #[test]
    fn canonical_json_has_sorted_keys() {
        let schema = Schema::new(DatabaseKind::PostgreSql);
        let json = schema.to_canonical_json().unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = value.as_object().unwrap();
        let keys: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted, "JSON keys must be alphabetically sorted");
    }

    #[test]
    fn many_to_many_implicit_preserves_junction() {
        use super::relation::Relation;

        let relation = Relation {
            name: Some("PostToTag".to_string()),
            kind: RelationKind::ManyToMany {
                junction: JunctionTable {
                    table: "_PostToTag".to_string(),
                    from_column: "A".to_string(),
                    to_column: "B".to_string(),
                },
                implicit: true,
            },
            from_table: "Post".to_string(),
            from_fields: vec!["id".to_string()],
            to_table: "Tag".to_string(),
            to_fields: vec!["id".to_string()],
            cascade: CascadeOptions { on_delete: None, on_update: None },
        };

        let json = serde_json::to_string(&relation).unwrap();
        let restored: Relation = serde_json::from_str(&json).unwrap();
        assert_eq!(relation, restored);

        // Junction must be present even for implicit M2M
        if let RelationKind::ManyToMany { junction, implicit } = &restored.kind {
            assert!(implicit);
            assert_eq!(junction.table, "_PostToTag");
        } else {
            panic!("expected ManyToMany");
        }
    }
}
