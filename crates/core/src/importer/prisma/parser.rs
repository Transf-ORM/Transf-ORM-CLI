use pest::error::{ErrorVariant, InputLocation};
use pest::iterators::Pair;
use pest::Span;
use pest_derive::Parser;

use super::ast::{RawArg, RawAttr, RawField, RawModel, RawTypeAlias, RawValue};
use crate::importer::{ImportError, SourceLocation};
use crate::pivot::{DatabaseKind, Enum, EnumValue};

#[derive(Parser)]
#[grammar = "src/importer/prisma/grammar.pest"]
pub(super) struct PrismaParser;

/// Converts a pest syntax error into a located, human-readable [`ImportError`].
pub(super) fn syntax_error(err: pest::error::Error<Rule>, input: &str) -> ImportError {
    let (start, end) = match err.location {
        InputLocation::Pos(p) => (p, p),
        InputLocation::Span(span) => span,
    };

    let found = describe_found(input, start);
    let message = match &err.variant {
        ErrorVariant::ParsingError {
            positives,
            negatives,
        } => {
            // `end of file` is always a valid alternative between top-level
            // blocks; listing it next to real suggestions is just noise.
            let positives: Vec<Rule> = match positives.as_slice() {
                [Rule::EOI] => vec![Rule::EOI],
                p => p.iter().copied().filter(|r| *r != Rule::EOI).collect(),
            };
            match (positives.is_empty(), negatives.is_empty()) {
                (false, _) => format!("expected {}, found {found}", describe_rules(&positives)),
                (true, false) => format!("unexpected {}", describe_rules(negatives)),
                (true, true) => format!("unexpected {found}"),
            }
        }
        ErrorVariant::CustomError { message } => message.clone(),
    };

    // When the parser stops on a symbol or at end of file at the start of a new
    // line, the problem is almost always at the end of the previous token
    // (missing type, unclosed block…), so point there instead — like rustc
    // does for a missing `;`. A misspelled word is reported where it is.
    let found_word = input[start..].starts_with(|c: char| c.is_alphanumeric() || c == '_');
    let (start, end) = match anchor_after_previous_token(input, start) {
        Some(anchor) if !found_word => (anchor, anchor),
        _ if found_word && end == start => (start, word_end(input, start)),
        _ => (start, end),
    };

    let mut error = ImportError::parse(message)
        .with_location(SourceLocation::from_byte_range(input, start, end));
    if found == "end of file" && has_unclosed_brace(input) {
        error = error.with_help("a `{` block is never closed; add the missing `}`");
    }
    error
}

/// Whether `input` opens more `{` than it closes, ignoring strings and comments.
fn has_unclosed_brace(input: &str) -> bool {
    let mut depth: i64 = 0;
    for line in input.lines() {
        let mut in_string = false;
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '\\' if in_string => {
                    chars.next();
                }
                '"' => in_string = !in_string,
                '/' if !in_string && chars.peek() == Some(&'/') => break,
                '{' if !in_string => depth += 1,
                '}' if !in_string => depth -= 1,
                _ => {}
            }
        }
    }
    depth > 0
}

/// If only whitespace including a line break separates `pos` from the previous
/// token, returns the byte offset right after that token.
fn anchor_after_previous_token(input: &str, pos: usize) -> Option<usize> {
    let before = &input[..pos];
    let token_end = before.trim_end().len();
    (token_end > 0 && before[token_end..].contains('\n')).then_some(token_end)
}

/// Byte offset of the end of the identifier-like word starting at `pos`.
fn word_end(input: &str, pos: usize) -> usize {
    input[pos..]
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .map_or(input.len(), |len| pos + len)
}

/// Describes the text found at `pos`: a word, a single symbol, or end of file.
fn describe_found(input: &str, pos: usize) -> String {
    let rest = &input[pos..];
    let Some(first) = rest.chars().next() else {
        return "end of file".to_string();
    };
    let token = if first.is_alphanumeric() || first == '_' {
        &input[pos..word_end(input, pos)]
    } else {
        &rest[..first.len_utf8()]
    };
    match token {
        "\n" | "\r" => "end of line".to_string(),
        t => format!("`{t}`"),
    }
}

/// Joins rule descriptions as `a, b or c`, without duplicates.
fn describe_rules(rules: &[Rule]) -> String {
    let mut names: Vec<&str> = Vec::new();
    for name in rules.iter().map(|r| rule_name(*r)) {
        if !names.contains(&name) {
            names.push(name);
        }
    }
    match names.split_last() {
        Some((last, [])) => (*last).to_string(),
        Some((last, rest)) => format!("{} or {last}", rest.join(", ")),
        None => "something else".to_string(),
    }
}

/// Human-readable name of a grammar rule, used in syntax error messages.
fn rule_name(rule: Rule) -> &'static str {
    match rule {
        Rule::EOI => "end of file",
        Rule::WHITESPACE => "whitespace",
        Rule::COMMENT => "a comment",
        Rule::identifier | Rule::base_type => "an identifier",
        Rule::string_lit | Rule::string_inner => "a string",
        Rule::escape_seq => "an escape sequence",
        Rule::integer => "an integer",
        Rule::float_num => "a float",
        Rule::schema | Rule::block => {
            "a block (`model`, `enum`, `view`, `type`, `datasource` or `generator`)"
        }
        Rule::datasource_block => "a `datasource` block",
        Rule::generator_block => "a `generator` block",
        Rule::config_field => "a `key = value` setting",
        Rule::config_value => "a setting value",
        Rule::env_func => "`env(...)`",
        Rule::model_block => "a `model` block",
        Rule::model_entry => "a field",
        Rule::field_decl => "a field",
        Rule::field_type | Rule::scalar_field_type | Rule::unsupported_type => "a field type",
        Rule::type_modifier => "a type modifier (`?` or `[]`)",
        Rule::field_attribute => "an attribute (`@...`)",
        Rule::block_attribute => "a block attribute (`@@...`)",
        Rule::attribute_path => "an attribute name",
        Rule::attribute_args => "attribute arguments",
        Rule::arg_list | Rule::argument | Rule::positional_arg => "an argument",
        Rule::named_arg => "a named argument",
        Rule::attr_value => "a value",
        Rule::array_value => "a list",
        Rule::function_call => "a function call",
        Rule::enum_block => "an `enum` block",
        Rule::enum_entry => "an enum value",
        Rule::enum_value => "an enum value",
        Rule::view_block => "a `view` block",
        Rule::type_alias => "a type alias",
    }
}

/// Builds a [`ImportErrorKind::Schema`](ImportErrorKind) error pointing at `span`.
fn schema_error_at(message: impl Into<String>, span: Span) -> ImportError {
    ImportError::schema(message).with_location(SourceLocation::from_byte_range(
        span.get_input(),
        span.start(),
        span.end(),
    ))
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
pub(super) fn parse_model_block(pair: Pair<Rule>) -> Result<RawModel, ImportError> {
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
pub(super) fn parse_type_alias(pair: Pair<Rule>) -> Result<RawTypeAlias, ImportError> {
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

fn parse_field_decl(pair: Pair<Rule>) -> Result<RawField, ImportError> {
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
fn parse_arg_list(arg_list: Pair<Rule>) -> Result<Vec<RawArg>, ImportError> {
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

pub(super) fn parse_attribute(pair: Pair<Rule>) -> Result<RawAttr, ImportError> {
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

fn parse_attr_value(pair: Pair<Rule>) -> Result<RawValue, ImportError> {
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
            .map_err(|e| {
                schema_error_at(
                    format!("invalid integer `{}`: {e}", inner.as_str()),
                    inner.as_span(),
                )
            }),
        Rule::float_num => inner
            .as_str()
            .parse::<f64>()
            .map(RawValue::Float)
            .map_err(|e| {
                schema_error_at(
                    format!("invalid float `{}`: {e}", inner.as_str()),
                    inner.as_span(),
                )
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
        r => Err(schema_error_at(
            format!("unexpected rule in attr_value: {r:?}"),
            inner.as_span(),
        )),
    }
}

pub(super) fn parse_enum_block(pair: Pair<Rule>) -> Result<Enum, ImportError> {
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
