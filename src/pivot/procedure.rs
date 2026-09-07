use serde::{Deserialize, Serialize};

use super::types::ColumnType;

/// Procedural language used to write a function or stored procedure body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum FunctionLanguage {
    /// Standard SQL — portable but limited control flow.
    Sql,
    /// PostgreSQL PL/pgSQL — full procedural language with loops and error handling.
    PlPgSql,
    /// PostgreSQL PL/Python.
    PlPython,
    /// PostgreSQL PL/Perl.
    PlPerl,
    /// JavaScript (e.g. PL/V8 in PostgreSQL, or Edge Functions in Supabase).
    JavaScript,
    Custom {
        name: String,
    },
}

/// PostgreSQL function volatility category — affects query planning and caching.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Volatility {
    /// May return different results for the same inputs — cannot be optimized away.
    Volatile,
    /// Returns the same results for the same inputs within a single transaction.
    Stable,
    /// Always returns the same results for the same inputs — may be pre-evaluated.
    Immutable,
}

/// Direction of a function parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ParamMode {
    /// Input parameter — passed by the caller.
    In,
    /// Output parameter — returned to the caller.
    Out,
    /// Both input and output.
    InOut,
    /// Variadic — accepts a variable number of arguments of the given type.
    Variadic,
}

/// A single parameter of a [`Function`] or [`StoredProcedure`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionParam {
    /// Parameter name — optional in SQL but required by most ORMs for named binding.
    pub name: Option<String>,
    pub mode: ParamMode,
    pub param_type: ColumnType,
    /// SQL expression used as the default when the caller omits this parameter.
    pub default: Option<String>,
}

/// A stored database function — returns a value or a result set.
///
/// Functions are first-class IR nodes because ORMs increasingly support calling
/// them directly (e.g. Prisma `queryRaw`, TypeORM query builder).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Function {
    pub name: String,
    pub db_schema: Option<String>,
    pub language: FunctionLanguage,
    pub parameters: Vec<FunctionParam>,
    /// Return type — `None` for functions declared `RETURNS VOID`.
    pub return_type: Option<ColumnType>,
    /// `true` when the function returns `SETOF` (a result set rather than a scalar).
    pub returns_set: bool,
    /// Source body of the function.
    pub body: String,
    pub volatility: Option<Volatility>,
}

/// A stored database procedure — like a function but does not return a value.
///
/// Procedures (introduced in PostgreSQL 11) are called with `CALL` rather than
/// `SELECT` and support transaction control inside the body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredProcedure {
    pub name: String,
    pub db_schema: Option<String>,
    pub language: FunctionLanguage,
    pub parameters: Vec<FunctionParam>,
    /// Source body of the procedure.
    pub body: String,
}

/// When the trigger fires relative to the triggering statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TriggerTiming {
    /// Fires before the row is modified — can alter the new row values.
    Before,
    /// Fires after the row is modified — sees the committed state.
    After,
    /// Fires instead of the statement — used on views to redirect writes.
    InsteadOf,
}

/// DML event that activates a trigger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TriggerEvent {
    Insert,
    Update,
    Delete,
    Truncate,
}

/// Whether the trigger fires once per affected row or once per statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ForEach {
    /// Fires once for every row affected by the statement.
    Row,
    /// Fires once per statement, regardless of how many rows are affected.
    Statement,
}

/// A database trigger — a function automatically called in response to a DML event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trigger {
    pub name: String,
    /// Table or view that owns this trigger.
    pub table: String,
    pub timing: TriggerTiming,
    pub events: Vec<TriggerEvent>,
    pub for_each: ForEach,
    /// Optional SQL boolean expression — the trigger only fires when this is true.
    pub condition: Option<String>,
    /// Name of the database function to call when the trigger fires.
    pub function: String,
}
