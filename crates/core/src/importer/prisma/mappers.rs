use super::ast::{RawArg, RawAttr, RawModel, RawValue};
use crate::pivot::relation::ReferentialAction;
use crate::pivot::table::{IndexColumn, IndexType, SortDirection};
use crate::pivot::types::{DbHint, DefaultFn, DefaultValue, LiteralValue, ScalarType};

// ── Duplication helpers ────────────────────────────────────────────────────

/// Extracts `db_name` (from `@@map`) and `db_schema` (from `@@schema`) from a raw model's
/// block attributes.
pub(super) fn extract_db_naming(raw: &RawModel) -> (Option<String>, Option<String>) {
    let db_name = raw
        .block_attr("map")
        .and_then(|a| a.positional(0))
        .and_then(|v| v.as_str().map(String::from));
    let db_schema = raw
        .block_attr("schema")
        .and_then(|a| a.positional(0))
        .and_then(|v| v.as_str().map(String::from));
    (db_name, db_schema)
}

/// Extracts a column list from `named("fields")` or `positional(0)`.
///
/// Used for `@@index`, `@@unique`, `@@id` and similar multi-column attributes.
pub(super) fn extract_columns_from_attr(attr: &RawAttr) -> Vec<String> {
    attr.named("fields")
        .or_else(|| attr.positional(0))
        .map(RawValue::as_string_array)
        .unwrap_or_default()
}

// ── Index helpers ──────────────────────────────────────────────────────────

/// Parse an index column descriptor from a Prisma attribute value.
///
/// Handles both a plain identifier (`email`) and a function-call form
/// (`email(sort: Desc)`) produced by `@@index([email(sort: Desc)])`.
pub(super) fn extract_one_index_col(v: &RawValue) -> IndexColumn {
    match v {
        RawValue::Ident(name) => IndexColumn {
            column: name.clone(),
            sort: SortDirection::Asc,
            nulls: None,
        },
        RawValue::Call(name, args) => {
            let sort = args
                .iter()
                .find_map(|a| match a {
                    RawArg::Named(k, v) if k == "sort" => v.as_ident().map(|s| match s {
                        "Desc" => SortDirection::Desc,
                        _ => SortDirection::Asc,
                    }),
                    _ => None,
                })
                .unwrap_or(SortDirection::Asc);
            IndexColumn {
                column: name.clone(),
                sort,
                nulls: None,
            }
        }
        _ => IndexColumn {
            column: String::new(),
            sort: SortDirection::Asc,
            nulls: None,
        },
    }
}

pub(super) fn extract_index_columns(v: &RawValue) -> Vec<IndexColumn> {
    match v {
        RawValue::Array(items) => items.iter().map(extract_one_index_col).collect(),
        other => vec![extract_one_index_col(other)],
    }
}

pub(super) fn map_index_type(s: &str) -> IndexType {
    match s {
        "BTree" | "Btree" => IndexType::BTree,
        "Hash" => IndexType::Hash,
        "Gin" => IndexType::Gin,
        "Gist" => IndexType::Gist,
        "Brin" => IndexType::Brin,
        "SpGist" => IndexType::SpGist,
        other => IndexType::Custom {
            name: other.to_string(),
        },
    }
}

// ── Type mapping helpers ───────────────────────────────────────────────────

pub(super) fn map_scalar_type(prisma_type: &str) -> Option<ScalarType> {
    Some(match prisma_type {
        "String" => ScalarType::String,
        "Int" => ScalarType::Int,
        "BigInt" => ScalarType::BigInt,
        // Prisma Float is 64-bit (equivalent to Double in our IR)
        "Float" => ScalarType::Double,
        "Decimal" => ScalarType::Decimal {
            precision: 65,
            scale: 30,
        },
        "Boolean" => ScalarType::Boolean,
        "DateTime" => ScalarType::Timestamp {
            precision: 3,
            with_timezone: false,
        },
        "Json" => ScalarType::Json,
        "Bytes" => ScalarType::Bytes,
        _ => return None,
    })
}

pub(super) fn map_db_hint(path: &str, args: &[RawArg]) -> Option<DbHint> {
    let hint = path.strip_prefix("db.")?;

    // Helper: first positional int argument.
    let first_int = || -> Option<i64> {
        args.iter().find_map(|a| match a {
            RawArg::Positional(v) => v.as_int(),
            RawArg::Named(_, v) => v.as_int(),
        })
    };

    // Helper: two positional ints (precision, scale) with named-arg fallback.
    let two_ints = || -> (Option<u8>, Option<u8>) {
        let named_p = args.iter().find_map(|a| match a {
            RawArg::Named(k, v) if k == "precision" => v.as_int().map(|n| n as u8),
            _ => None,
        });
        let named_s = args.iter().find_map(|a| match a {
            RawArg::Named(k, v) if k == "scale" => v.as_int().map(|n| n as u8),
            _ => None,
        });
        let mut pos = args.iter().filter_map(|a| match a {
            RawArg::Positional(v) => v.as_int().map(|n| n as u8),
            _ => None,
        });
        let p = named_p.or_else(|| pos.next());
        let s = named_s.or_else(|| pos.next());
        (p, s)
    };

    // Helper: optional precision arg (defaults to 0 when omitted, e.g. `@db.Time`).
    let precision = || first_int().unwrap_or(0) as u8;

    Some(match hint {
        // ── String ───────────────────────────────────────────────────────
        "VarChar" => DbHint::VarChar {
            length: first_int()? as u32,
        },
        "Char" => DbHint::Char {
            length: first_int()? as u32,
        },
        "Text" => DbHint::Text,
        "TinyText" => DbHint::TinyText,
        "MediumText" => DbHint::MediumText,
        "LongText" => DbHint::LongText,
        // ── Integer ──────────────────────────────────────────────────────
        "SmallInt" => DbHint::SmallInt,
        "Int" | "Integer" => DbHint::Int,
        "BigInt" => DbHint::BigInt,
        "TinyInt" => DbHint::TinyInt,
        "MediumInt" => DbHint::MediumInt,
        "Year" => DbHint::Year,
        // ── Float ────────────────────────────────────────────────────────
        "Float" => DbHint::Float,
        "DoublePrecision" => DbHint::DoublePrecision,
        "Real" => DbHint::Real,
        "Decimal" | "Numeric" => {
            let (p, s) = two_ints();
            DbHint::Numeric {
                precision: p?,
                scale: s?,
            }
        }
        // ── Binary / blob ─────────────────────────────────────────────────
        "VarBinary" => DbHint::VarBinary {
            length: first_int()? as u32,
        },
        "Binary" => DbHint::Binary {
            length: first_int()? as u32,
        },
        "TinyBlob" => DbHint::TinyBlob,
        "Blob" => DbHint::Blob,
        "MediumBlob" => DbHint::MediumBlob,
        "LongBlob" => DbHint::LongBlob,
        "ByteA" => DbHint::ByteA,
        "Bit" | "VarBit" => DbHint::Bit {
            length: first_int().map(|n| n as u32),
        },
        // ── Date / time ──────────────────────────────────────────────────
        "Date" => DbHint::Date,
        "Time" => DbHint::Time {
            precision: precision(),
        },
        "DateTime" => DbHint::DateTime {
            precision: precision(),
        },
        "Timestamp" => DbHint::Timestamp {
            precision: precision(),
        },
        "Timestamptz" => DbHint::Timestamptz,
        "Timetz" => DbHint::Timetz,
        "Interval" => DbHint::Interval,
        // ── PostgreSQL-specific ───────────────────────────────────────────
        "Uuid" => DbHint::Uuid,
        "Inet" => DbHint::Inet,
        "Cidr" => DbHint::Cidr,
        "Xml" => DbHint::Xml,
        "Money" => DbHint::Money,
        "Oid" => DbHint::Oid,
        other => DbHint::Custom {
            type_expr: other.to_string(),
        },
    })
}

pub(super) fn map_default_value(v: &RawValue) -> Option<DefaultValue> {
    match v {
        RawValue::Str(s) => Some(DefaultValue::Literal(LiteralValue::String(s.clone()))),
        RawValue::Int(n) => Some(DefaultValue::Literal(LiteralValue::Int(*n))),
        RawValue::Float(f) => Some(DefaultValue::Literal(LiteralValue::Float(*f))),
        RawValue::Ident(s) => match s.as_str() {
            "true" => Some(DefaultValue::Literal(LiteralValue::Bool(true))),
            "false" => Some(DefaultValue::Literal(LiteralValue::Bool(false))),
            other => Some(DefaultValue::Literal(LiteralValue::String(
                other.to_string(),
            ))),
        },
        RawValue::Call(name, args) => match name.as_str() {
            "autoincrement" => Some(DefaultValue::Function(DefaultFn::AutoIncrement)),
            "now" => Some(DefaultValue::Function(DefaultFn::Now)),
            "uuid" => Some(DefaultValue::Function(DefaultFn::Uuid)),
            "cuid" => Some(DefaultValue::Function(DefaultFn::Cuid)),
            "cuid2" => Some(DefaultValue::Function(DefaultFn::Cuid2)),
            "ulid" => Some(DefaultValue::Function(DefaultFn::Ulid)),
            "dbgenerated" => {
                let expr = args
                    .iter()
                    .find_map(|a| match a {
                        RawArg::Positional(v) => v.as_str().map(String::from),
                        _ => None,
                    })
                    .unwrap_or_default();
                Some(DefaultValue::Raw(expr))
            }
            _ => None,
        },
        RawValue::Array(_) => None,
    }
}

pub(super) fn parse_referential_action(s: &str) -> Option<ReferentialAction> {
    Some(match s {
        "Cascade" => ReferentialAction::Cascade,
        "Restrict" => ReferentialAction::Restrict,
        "SetNull" => ReferentialAction::SetNull,
        "SetDefault" => ReferentialAction::SetDefault,
        "NoAction" => ReferentialAction::NoAction,
        _ => return None,
    })
}

// ── Unknown attribute helpers ──────────────────────────────────────────────

pub(super) fn raw_value_to_json(v: &RawValue) -> serde_json::Value {
    match v {
        RawValue::Str(s) => serde_json::Value::String(s.clone()),
        RawValue::Int(n) => serde_json::json!(n),
        RawValue::Float(f) => serde_json::json!(f),
        RawValue::Ident(s) => serde_json::Value::String(s.clone()),
        RawValue::Array(items) => {
            serde_json::Value::Array(items.iter().map(raw_value_to_json).collect())
        }
        RawValue::Call(name, args) => serde_json::json!({
            "fn": name,
            "args": args.iter().map(|a| match a {
                RawArg::Named(k, v)   => serde_json::json!({ k: raw_value_to_json(v) }),
                RawArg::Positional(v) => raw_value_to_json(v),
            }).collect::<Vec<_>>(),
        }),
    }
}

pub(super) fn raw_args_to_json(args: &[RawArg]) -> serde_json::Value {
    if args.is_empty() {
        return serde_json::Value::Bool(true);
    }
    let positional: Vec<serde_json::Value> = args
        .iter()
        .filter_map(|a| match a {
            RawArg::Positional(v) => Some(raw_value_to_json(v)),
            _ => None,
        })
        .collect();
    if positional.len() == 1 && args.iter().all(|a| matches!(a, RawArg::Positional(_))) {
        return positional.into_iter().next().unwrap();
    }
    let mut obj = serde_json::Map::new();
    for arg in args {
        match arg {
            RawArg::Named(k, v) => {
                obj.insert(k.clone(), raw_value_to_json(v));
            }
            RawArg::Positional(v) => {
                obj.insert("_".to_string(), raw_value_to_json(v));
            }
        }
    }
    serde_json::Value::Object(obj)
}
