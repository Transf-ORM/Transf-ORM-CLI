use serde::{Deserialize, Serialize};

use super::metadata::OrmMetadata;
use super::relation::{ForeignKey, Relation};
use super::types::{ColumnType, DefaultValue};

/// A single column in a table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    /// Logical name exposed by the ORM.
    pub name: String,
    /// Actual column name in the database when it differs from `name` (e.g. Prisma `@map`).
    pub db_name: Option<String>,
    #[serde(rename = "type")]
    pub col_type: ColumnType,
    pub nullable: bool,
    pub default: Option<DefaultValue>,
}

/// Primary key constraint on a table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrimaryKey {
    /// Constraint name as defined in the database schema.
    pub name: Option<String>,
    /// Ordered list of column names that form the primary key.
    pub columns: Vec<String>,
}

/// Sort direction for an index column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Where NULL values appear relative to non-NULL values in an index.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NullsPosition {
    First,
    Last,
}

/// A single column entry inside an [`Index`], with its sort options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexColumn {
    pub column: String,
    pub sort: SortDirection,
    pub nulls: Option<NullsPosition>,
}

/// Index access method used by the database engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum IndexType {
    /// Default B-tree index — suitable for equality and range queries.
    BTree,
    /// Hash index — equality-only, faster than B-tree for exact matches.
    Hash,
    /// PostgreSQL GIN — inverted index for JSONB, arrays, and full-text search.
    Gin,
    /// PostgreSQL GiST — generalized search tree for geometric and full-text types.
    Gist,
    /// PostgreSQL BRIN — block range index, efficient for naturally ordered large tables.
    Brin,
    /// PostgreSQL SP-GiST — space-partitioned generalized search tree.
    SpGist,
    Custom {
        name: String,
    },
}

/// A database index on one or more columns of a table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Index {
    pub name: Option<String>,
    pub columns: Vec<IndexColumn>,
    pub unique: bool,
    /// SQL `WHERE` clause for a partial index — only rows matching the condition are indexed.
    pub partial: Option<String>,
    pub index_type: Option<IndexType>,
    /// Additional columns included in the index leaf pages (PostgreSQL `INCLUDE`).
    ///
    /// Useful for covering indexes — the included columns satisfy queries without
    /// a heap fetch but are not part of the sort key.
    pub include: Vec<String>,
}

/// Unique constraint on a set of columns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniqueConstraint {
    pub name: Option<String>,
    pub columns: Vec<String>,
}

/// Check constraint — a boolean SQL expression that every row must satisfy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckConstraint {
    pub name: Option<String>,
    /// SQL boolean expression evaluated for each row on insert and update.
    pub expression: String,
}

/// Common ORM behavioral patterns modeled as first-class IR concepts.
///
/// Storing these explicitly (rather than in [`OrmMetadata`])
/// lets exporters map them to the correct ORM-specific annotation without
/// needing to parse raw metadata — e.g. `CreatedAt` becomes `@CreateDateColumn`
/// in TypeORM and `default: sql\`now()\`` in Drizzle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Behavior {
    /// Soft-delete pattern — rows are marked deleted rather than physically removed.
    SoftDelete { column: String },
    /// Column automatically set to the current timestamp on insert.
    CreatedAt { column: String },
    /// Column automatically updated to the current timestamp on every update.
    UpdatedAt { column: String },
    /// Optimistic locking version column — incremented on each update to detect conflicts.
    Versioning { column: String },
}

/// Table inheritance strategy — used by TypeORM, Hibernate, Entity Framework, and others.
///
/// - **SingleTable**: all subclass columns in one table, discriminated by a column.
/// - **TablePerClass**: each concrete class has its own table with all inherited columns.
/// - **ConcreteTable**: like TablePerClass, but abstract classes also have a table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "strategy", rename_all = "camelCase")]
pub enum Inheritance {
    SingleTable {
        discriminator_column: String,
        /// Value in the discriminator column that identifies rows of this class.
        discriminator_value: Option<String>,
    },
    TablePerClass {
        parent_table: Option<String>,
    },
    ConcreteTable {
        parent_table: Option<String>,
    },
}

/// A database table — the central node of the pivot IR.
///
/// Holds both the **database layer** (columns, indexes, constraints, foreign keys)
/// and the **ORM layer** (relations, behaviors, inheritance) so that importers and
/// exporters can work from a single coherent structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Table {
    /// Logical name as exposed by the ORM.
    pub name: String,
    /// Actual table name in the database when it differs from `name` (e.g. Prisma `@@map`).
    pub db_name: Option<String>,
    /// PostgreSQL schema (or MySQL database) that owns this table.
    pub db_schema: Option<String>,
    pub columns: Vec<Column>,
    pub primary_key: Option<PrimaryKey>,
    pub indexes: Vec<Index>,
    pub unique_constraints: Vec<UniqueConstraint>,
    pub check_constraints: Vec<CheckConstraint>,
    /// Database-level foreign key constraints.
    pub foreign_keys: Vec<ForeignKey>,
    /// ORM-level relations — see [`Relation`] for the distinction from foreign keys.
    pub relations: Vec<Relation>,
    pub inheritance: Option<Inheritance>,
    pub behaviors: Vec<Behavior>,
    pub metadata: OrmMetadata,
}
