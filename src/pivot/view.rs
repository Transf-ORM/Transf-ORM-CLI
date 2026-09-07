use serde::{Deserialize, Serialize};

use super::metadata::OrmMetadata;
use super::table::Index;

/// A typed column descriptor for a view.
///
/// Views do not enforce column types at the database level, but ORMs
/// and code generators may expose typed column metadata for query building.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewColumn {
    pub name: String,
    /// Actual column name in the view definition when it differs from `name`.
    pub db_name: Option<String>,
}

/// Refresh strategy for a [`MaterializedView`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RefreshStrategy {
    /// Refresh must be triggered explicitly with `REFRESH MATERIALIZED VIEW`.
    Manual,
    /// PostgreSQL `CONCURRENTLY` — refreshes without locking reads, requires a unique index.
    Concurrent,
    /// Full recomputation — locks the view during refresh.
    Complete,
}

/// A database view — a named SQL query stored in the database.
///
/// Views are first-class IR nodes because several ORMs (Prisma, Drizzle, TypeORM)
/// support mapping models or entities directly to views.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub name: String,
    pub db_name: Option<String>,
    /// PostgreSQL schema (or MySQL database) that owns this view.
    pub db_schema: Option<String>,
    /// SQL `SELECT` statement that defines the view.
    pub definition: String,
    pub columns: Vec<ViewColumn>,
    /// Whether the view supports `INSERT` and `UPDATE` statements.
    pub updatable: bool,
    pub metadata: OrmMetadata,
}

/// A materialized view — a view whose result set is physically stored and refreshed on demand.
///
/// Unlike regular [`View`]s, materialized views can have indexes because their
/// data lives on disk. Refresh behavior is controlled by [`RefreshStrategy`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterializedView {
    pub name: String,
    pub db_name: Option<String>,
    pub db_schema: Option<String>,
    /// SQL `SELECT` statement that defines the view.
    pub definition: String,
    pub columns: Vec<ViewColumn>,
    /// Indexes on the materialized view's stored data.
    pub indexes: Vec<Index>,
    pub refresh_strategy: Option<RefreshStrategy>,
    pub metadata: OrmMetadata,
}
