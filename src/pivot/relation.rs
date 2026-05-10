use serde::{Deserialize, Serialize};

/// Action taken on related rows when the referenced row is deleted or updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReferentialAction {
    /// Delete or update the child rows automatically.
    Cascade,
    /// Prevent deletion or update if child rows exist.
    Restrict,
    /// Set the foreign key column to NULL.
    SetNull,
    /// Set the foreign key column to its default value.
    SetDefault,
    /// Take no action — the database enforces nothing.
    NoAction,
}

/// Delete and update actions for a foreign key or relation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeOptions {
    pub on_delete: Option<ReferentialAction>,
    pub on_update: Option<ReferentialAction>,
}

/// Database-level foreign key constraint — what actually exists in the database DDL.
///
/// Distinct from [`Relation`], which is the ORM abstraction on top. Some ORMs
/// (e.g. Drizzle) define relations independently of foreign keys; others (e.g. Prisma)
/// derive them from the same declaration. Keeping them separate in the IR ensures
/// both layers are preserved across conversions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForeignKey {
    /// Constraint name as it appears in the database (may be database-generated).
    pub name: Option<String>,
    /// Columns on the owning table that hold the foreign key values.
    pub columns: Vec<String>,
    pub referenced_table: String,
    pub referenced_columns: Vec<String>,
    pub on_delete: Option<ReferentialAction>,
    pub on_update: Option<ReferentialAction>,
}

/// The junction (join) table for a many-to-many relation.
///
/// Always populated even when [`RelationKind::ManyToMany::implicit`] is `true`,
/// so that round-trips back to the source ORM can reconstruct the original
/// implicit declaration without information loss.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JunctionTable {
    pub table: String,
    /// Column in the junction table pointing to the `from` side.
    pub from_column: String,
    /// Column in the junction table pointing to the `to` side.
    pub to_column: String,
}

/// Cardinality and structure of an ORM-level relation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RelationKind {
    /// One-to-one — `owner_table` holds the foreign key.
    OneToOne { owner_table: String },
    OneToMany,
    ManyToMany {
        junction: JunctionTable,
        /// `true` — the ORM manages the junction table invisibly (Prisma implicit M2M).
        ///
        /// `false` — the junction table is an explicit [`Table`](super::table::Table)
        /// in the schema and must be present in [`Schema::tables`](super::Schema::tables).
        implicit: bool,
    },
}

/// ORM-level relation — the abstraction the ORM adds on top of a [`ForeignKey`].
///
/// Kept separate from [`ForeignKey`] because some ORMs (Drizzle, MikroORM) define
/// relations independently of the database schema, while others (Prisma, TypeORM)
/// derive both from the same model declaration.
///
/// # Examples
///
/// ```
/// use transf_orm_cli::pivot::relation::{
///     CascadeOptions, JunctionTable, Relation, RelationKind,
/// };
///
/// // Prisma implicit many-to-many between Post and Tag
/// let rel = Relation {
///     name: Some("PostToTag".to_string()),
///     kind: RelationKind::ManyToMany {
///         junction: JunctionTable {
///             table: "_PostToTag".to_string(),
///             from_column: "A".to_string(),
///             to_column: "B".to_string(),
///         },
///         implicit: true,
///     },
///     from_table: "Post".to_string(),
///     from_fields: vec!["id".to_string()],
///     to_table: "Tag".to_string(),
///     to_fields: vec!["id".to_string()],
///     cascade: CascadeOptions { on_delete: None, on_update: None },
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relation {
    /// Optional relation name — required by some ORMs when multiple relations
    /// exist between the same two tables.
    pub name: Option<String>,
    pub kind: RelationKind,
    pub from_table: String,
    pub from_fields: Vec<String>,
    pub to_table: String,
    pub to_fields: Vec<String>,
    pub cascade: CascadeOptions,
}
