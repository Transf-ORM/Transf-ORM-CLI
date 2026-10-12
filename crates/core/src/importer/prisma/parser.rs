use pest::iterators::Pair;
use pest_derive::Parser;

use super::ast::{RawArg, RawAttr, RawField, RawModel, RawTypeAlias, RawValue};
use crate::pivot::{DatabaseKind, Enum, EnumValue};

#[derive(Parser)]
#[grammar = "src/importer/prisma/grammar.pest"]
pub(super) struct PrismaParser;

impl From<pest::error::Error<Rule>> for super::super::ImportError {
    fn from(e: pest::error::Error<Rule>) -> Self {
        super::super::ImportError::Parse(e.to_string())
    }
}

pub(super) fn extract_database_kind(pair: Pair<Rule>) -> DatabaseKind {
    for child in pair.into_inner() {
        if child.as_rule() != Rule::config_field {
            continue;
        }
        let mut cf = child.into_inner();
        if cf.next().map(|p| p.as_str()).unwrap_or("") != "provider" {
            continue;
        }
        if let Some(val_pair) = cf.next() {
            if let Some(inner) = val_pair.into_inner().next() {
                let provider = match inner.as_rule() {
                    Rule::string_lit => inner.into_inner().next().map(|s| s.as_str()).unwrap_or(""),
                    Rule::identifier => inner.as_str(),
                    _ => continue,
                };
                return match provider {
                    "postgresql" | "postgres" => DatabaseKind::PostgreSql,
                    "mysql" => DatabaseKind::MySql,
                    "sqlite" => DatabaseKind::Sqlite,
                    "sqlserver" => DatabaseKind::MsSql,
                    "cockroachdb" => DatabaseKind::CockroachDb,
                    other => DatabaseKind::Custom(other.to_string()),
                };
            }
        }
    }
    DatabaseKind::PostgreSql
}

/// Parse a `model_block` or `view_block` (same inner structure) into a `RawModel`.
pub(super) fn parse_model_block(pair: Pair<Rule>) -> Result<RawModel, super::super::ImportError> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let mut fields = Vec::new();
    let mut block_attrs = Vec::new();

    for entry in inner {
        let child = entry.into_inner().next().unwrap();
        match child.as_rule() {
            Rule::field_decl => fields.push(parse_field_decl(child)?),
            Rule::block_attribute => block_attrs.push(parse_attribute(child)?),
            _ => {}
        }
    }
    Ok(RawModel {
        name,
        fields,
        block_attrs,
    })
}

/// Parse a `type` alias block: `type Name = BaseType @attr...`
pub(super) fn parse_type_alias(
    pair: Pair<Rule>,
) -> Result<RawTypeAlias, super::super::ImportError> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let base_type = inner.next().unwrap().as_str().to_string();
    let attrs: Result<Vec<_>, _> = inner.map(parse_attribute).collect();
    Ok(RawTypeAlias {
        name,
        base_type,
        attrs: attrs?,
    })
}

fn parse_field_decl(pair: Pair<Rule>) -> Result<RawField, super::super::ImportError> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let (base_type, unsupported_type, is_optional, is_array) =
        parse_field_type(inner.next().unwrap());
    let attrs: Result<Vec<_>, _> = inner.map(parse_attribute).collect();
    Ok(RawField {
        name,
        base_type,
        unsupported_type,
        is_optional,
        is_array,
        attrs: attrs?,
    })
}

/// Returns (base_type, unsupported_raw, is_optional, is_array).
fn parse_field_type(pair: Pair<Rule>) -> (String, Option<String>, bool, bool) {
    let child = pair.into_inner().next().unwrap();
    match child.as_rule() {
        Rule::unsupported_type => {
            let mut ut = child.into_inner();
            let raw = ut
                .next()
                .unwrap() // string_lit
                .into_inner()
                .next()
                .unwrap() // string_inner
                .as_str()
                .to_string();
            let modifier = ut.next().map(|m| m.as_str().to_string());
            let is_opt = modifier.as_deref() == Some("?");
            let is_arr = modifier.as_deref() == Some("[]");
            (String::new(), Some(raw), is_opt, is_arr)
        }
        Rule::scalar_field_type => {
            let mut sft = child.into_inner();
            let base = sft.next().unwrap().as_str().to_string();
            let modifier = sft.next().map(|m| m.as_str().to_string());
            let is_opt = modifier.as_deref() == Some("?");
            let is_arr = modifier.as_deref() == Some("[]");
            (base, None, is_opt, is_arr)
        }
        _ => (String::new(), None, false, false),
    }
}

/// Parse an `arg_list` pair into a list of raw arguments.
fn parse_arg_list(arg_list: Pair<Rule>) -> Result<Vec<RawArg>, super::super::ImportError> {
    let mut args = Vec::new();
    for arg_pair in arg_list.into_inner() {
        let child = arg_pair.into_inner().next().unwrap();
        match child.as_rule() {
            Rule::named_arg => {
                let mut na = child.into_inner();
                let key = na.next().unwrap().as_str().to_string();
                let val = parse_attr_value(na.next().unwrap())?;
                args.push(RawArg::Named(key, val));
            }
            Rule::positional_arg => {
                let val = parse_attr_value(child.into_inner().next().unwrap())?;
                args.push(RawArg::Positional(val));
            }
            _ => {}
        }
    }
    Ok(args)
}

pub(super) fn parse_attribute(pair: Pair<Rule>) -> Result<RawAttr, super::super::ImportError> {
    let mut inner = pair.into_inner();
    let path = inner.next().unwrap().as_str().to_string();
    let args = if let Some(args_pair) = inner.next() {
        if let Some(arg_list) = args_pair.into_inner().next() {
            parse_arg_list(arg_list)?
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    Ok(RawAttr { path, args })
}

fn parse_attr_value(pair: Pair<Rule>) -> Result<RawValue, super::super::ImportError> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::string_lit => {
            let s = inner.into_inner().next().unwrap().as_str().to_string();
            Ok(RawValue::Str(s))
        }
        Rule::integer => inner
            .as_str()
            .parse::<i64>()
            .map(RawValue::Int)
            .map_err(|_| {
                super::super::ImportError::Schema(format!("invalid integer: {}", inner.as_str()))
            }),
        Rule::float_num => inner
            .as_str()
            .parse::<f64>()
            .map(RawValue::Float)
            .map_err(|_| {
                super::super::ImportError::Schema(format!("invalid float: {}", inner.as_str()))
            }),
        Rule::identifier => Ok(RawValue::Ident(inner.as_str().to_string())),
        Rule::array_value => {
            let items: Result<Vec<_>, _> = inner.into_inner().map(parse_attr_value).collect();
            Ok(RawValue::Array(items?))
        }
        Rule::function_call => {
            let mut fc = inner.into_inner();
            let name = fc.next().unwrap().as_str().to_string();
            let args = if let Some(arg_list) = fc.next() {
                parse_arg_list(arg_list)?
            } else {
                Vec::new()
            };
            Ok(RawValue::Call(name, args))
        }
        r => Err(super::super::ImportError::Schema(format!(
            "unexpected rule in attr_value: {:?}",
            r
        ))),
    }
}

pub(super) fn parse_enum_block(pair: Pair<Rule>) -> Result<Enum, super::super::ImportError> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let mut db_name: Option<String> = None;
    let mut db_schema: Option<String> = None;
    let mut values: Vec<EnumValue> = Vec::new();

    for entry in inner {
        let child = entry.into_inner().next().unwrap();
        match child.as_rule() {
            Rule::block_attribute => {
                let attr = parse_attribute(child)?;
                match attr.path.as_str() {
                    "map" => {
                        db_name = attr
                            .positional(0)
                            .and_then(|v| v.as_str().map(String::from))
                    }
                    "schema" => {
                        db_schema = attr
                            .positional(0)
                            .and_then(|v| v.as_str().map(String::from))
                    }
                    _ => {}
                }
            }
            Rule::enum_value => {
                let mut ev = child.into_inner();
                let val_name = ev.next().unwrap().as_str().to_string();
                let mut val_db_name = None;
                for attr_pair in ev {
                    let attr = parse_attribute(attr_pair)?;
                    if attr.path == "map" {
                        val_db_name = attr
                            .positional(0)
                            .and_then(|v| v.as_str().map(String::from));
                    }
                }
                values.push(EnumValue {
                    name: val_name,
                    db_name: val_db_name,
                });
            }
            _ => {}
        }
    }
    Ok(Enum {
        name,
        db_name,
        db_schema,
        values,
    })
}
