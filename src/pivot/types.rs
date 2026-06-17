use serde::{Deserialize, Serialize};

/// Abstract scalar type — database-agnostic representation of a column's value kind.
///
/// Paired with an optional [`DbHint`] inside [`ColumnType`] to preserve
/// database-specific precision (e.g. `String` + `DbHint::VarChar { length: 255 }`).
/// Without a hint, exporters fall back to a sensible default for the target database.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ScalarType {
    String,
    Int,
    SmallInt,
    BigInt,
    Float,
    Double,
    Decimal {
        precision: u8,
        scale: u8,
    },
    Boolean,
    Date,
    Time {
        precision: u8,
    },
    DateTime {
        precision: u8,
    },
    Timestamp {
        precision: u8,
        with_timezone: bool,
    },
    Json,
    /// PostgreSQL JSONB — binary JSON with indexing support.
    JsonB,
    Uuid,
    Bytes,
    Xml,
    /// PostgreSQL INET — IPv4 or IPv6 host address.
    Inet,
    /// PostgreSQL CIDR — IPv4 or IPv6 network address.
    Cidr,
    MacAddr,
    /// PostgreSQL TSVECTOR — pre-processed full-text search document.
    TsVector,
    /// PostgreSQL TSQUERY — full-text search query.
    TsQuery,
    /// Reference to a named [`Enum`](crate::pivot::Enum) defined in the same schema.
    Enum {
        name: String,
    },
    /// Raw database type with no known IR equivalent.
    ///
    /// Preserved verbatim so round-trips back to the source ORM are lossless.
    /// Exporters targeting a different ORM must decide how to handle this — usually
    /// by emitting a warning and carrying the raw expression through.
    Unsupported {
        type_name: String,
    },
}

/// Database-specific type hint layered on top of [`ScalarType`].
///
/// Required to avoid lossy conversions: Prisma represents `@db.VarChar(255)` as
/// `ScalarType::String` + `DbHint::VarChar { length: 255 }`. Without the hint,
/// the exporter would emit a plain `text` column and lose the length constraint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum DbHint {
    // ── String types ───────────────────────────────────────────────────────
    VarChar {
        length: u32,
    },
    Char {
        length: u32,
    },
    Text,
    TinyText,
    MediumText,
    LongText,
    // ── Integer types ──────────────────────────────────────────────────────
    /// Explicit SMALLINT / INT2 column type hint (e.g. `@db.SmallInt`).
    SmallInt,
    /// Explicit INT / INT4 column type hint (e.g. MySQL `@db.Int`).
    Int,
    /// Explicit BIGINT / INT8 column type hint (e.g. `@db.BigInt`).
    BigInt,
    TinyInt,
    MediumInt,
    Year,
    // ── Floating point ─────────────────────────────────────────────────────
    /// Single-precision FLOAT (4 bytes).
    Float,
    /// DOUBLE PRECISION / FLOAT8 (8 bytes).
    DoublePrecision,
    /// REAL — alias for single-precision float in PostgreSQL.
    Real,
    Numeric {
        precision: u8,
        scale: u8,
    },
    // ── Binary / blob types ────────────────────────────────────────────────
    VarBinary {
        length: u32,
    },
    Binary {
        length: u32,
    },
    TinyBlob,
    Blob,
    MediumBlob,
    LongBlob,
    /// PostgreSQL BYTEA — variable-length binary string.
    ByteA,
    /// BIT(n) or VARBIT(n) — fixed or variable-length bit string.
    Bit {
        length: Option<u32>,
    },
    // ── Date / time ────────────────────────────────────────────────────────
    /// DATE — calendar date without time component.
    Date,
    /// TIME(n) — time of day without date.
    Time {
        precision: u8,
    },
    /// MySQL DATETIME(n) — date and time without timezone.
    DateTime {
        precision: u8,
    },
    /// MySQL TIMESTAMP(n) or PostgreSQL TIMESTAMP(n) without timezone.
    Timestamp {
        precision: u8,
    },
    /// PostgreSQL TIMESTAMPTZ shorthand.
    Timestamptz,
    /// PostgreSQL TIMETZ shorthand.
    Timetz,
    Interval,
    // ── PostgreSQL-specific ────────────────────────────────────────────────
    /// PostgreSQL UUID column type (as opposed to storing UUID as TEXT).
    Uuid,
    /// PostgreSQL INET — IPv4 or IPv6 host address.
    Inet,
    /// PostgreSQL CIDR — network address.
    Cidr,
    /// PostgreSQL XML.
    Xml,
    /// PostgreSQL MONEY.
    Money,
    /// PostgreSQL OID.
    Oid,
    // ── Catch-all ──────────────────────────────────────────────────────────
    /// Arbitrary database type expression not covered by other variants.
    Custom {
        type_expr: String,
    },
}

/// Full type descriptor for a column: abstract kind + optional DB-specific hint.
///
/// # Examples
///
/// ```
/// use transf_orm_cli::pivot::types::{ColumnType, DbHint, ScalarType};
///
/// // Prisma: String @db.VarChar(255)
/// let col = ColumnType {
///     scalar: ScalarType::String,
///     db_hint: Some(DbHint::VarChar { length: 255 }),
///     array: false,
/// };
///
/// // Simple int with no DB hint
/// let id = ColumnType::simple(ScalarType::Int);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnType {
    pub scalar: ScalarType,
    pub db_hint: Option<DbHint>,
    /// Whether this is a PostgreSQL array of `scalar` (e.g. `INT[]`).
    pub array: bool,
}

impl ColumnType {
    /// Shorthand for a plain scalar column with no DB hint and no array wrapper.
    pub fn simple(scalar: ScalarType) -> Self {
        Self {
            scalar,
            db_hint: None,
            array: false,
        }
    }
}

/// Concrete literal value used in a [`DefaultValue::Literal`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum LiteralValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

/// Database function called to compute a column's default value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DefaultFn {
    Now,
    Uuid,
    Cuid,
    Cuid2,
    Ulid,
    /// Auto-incrementing integer sequence.
    ///
    /// Encoded as a `DefaultValue`, not as a type, to unify Prisma's
    /// `@default(autoincrement())` and Drizzle's `serial()` at the IR level.
    AutoIncrement,
    /// Custom function call not covered by other variants.
    Custom {
        expr: String,
    },
}

/// Default value assigned to a column when no explicit value is provided on insert.
///
/// # Examples
///
/// ```
/// use transf_orm_cli::pivot::types::{DefaultFn, DefaultValue, LiteralValue};
///
/// let created_at = DefaultValue::Function(DefaultFn::Now);
/// let status     = DefaultValue::Literal(LiteralValue::String("active".to_string()));
/// let raw        = DefaultValue::Raw("gen_random_uuid()".to_string());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum DefaultValue {
    Literal(LiteralValue),
    Function(DefaultFn),
    /// Arbitrary SQL expression — escape hatch for defaults the IR cannot model.
    Raw(String),
}
