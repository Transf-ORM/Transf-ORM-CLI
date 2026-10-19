use std::collections::{HashMap, HashSet};

use pest::Parser;

use crate::pivot::{DatabaseKind, Enum, Schema};

mod ast;
mod behaviors;
mod mappers;
mod parser;
mod resolver;

use ast::{RawModel, RawTypeAlias};
use parser::{PrismaParser, Rule};

/// Importer for Prisma Schema Language (`.prisma`) files.
pub struct PrismaImporter;

impl super::Importer for PrismaImporter {
    fn import(&self, input: &str) -> Result<crate::pivot::Schema, super::ImportError> {
        parse_schema(input)
    }
}

/// Parse a Prisma Schema Language string and return the pivot [`Schema`].
///
/// # Errors
///
/// Returns [`ImportError::Parse`] if the input is not valid PSL, or
/// [`ImportError::Schema`] if the schema is structurally invalid (e.g. a
/// field references a model that does not exist).
///
/// # Examples
///
/// ```
/// use transf_orm_core::importer::prisma::parse_schema;
///
/// let input = r#"
///   datasource db { provider = "postgresql" url = env("DATABASE_URL") }
///   model User {
///     id    Int    @id @default(autoincrement())
///     email String @unique
///   }
/// "#;
///
/// let schema = parse_schema(input).unwrap();
/// assert_eq!(schema.tables.len(), 1);
/// assert_eq!(schema.tables[0].name, "User");
/// ```
pub fn parse_schema(input: &str) -> Result<Schema, super::ImportError> {
    let pairs = PrismaParser::parse(Rule::schema, input)?;

    let schema_pair = pairs
        .into_iter()
        .next()
        .expect("pest always produces at least one pair for a valid schema rule");

    let mut database = DatabaseKind::PostgreSql;
    let mut raw_models: Vec<RawModel> = Vec::new();
    let mut raw_views: Vec<RawModel> = Vec::new();
    let mut enums: Vec<Enum> = Vec::new();
    let mut type_aliases: HashMap<String, RawTypeAlias> = HashMap::new();

    for pair in schema_pair.into_inner() {
        match pair.as_rule() {
            Rule::block => {
                let inner = pair
                    .into_inner()
                    .next()
                    .expect("a block pair always has an inner rule");

                match inner.as_rule() {
                    Rule::datasource_block => database = parser::extract_database_kind(inner),
                    Rule::model_block => raw_models.push(parser::parse_model_block(inner)?),
                    Rule::view_block => raw_views.push(parser::parse_model_block(inner)?),
                    Rule::enum_block => enums.push(parser::parse_enum_block(inner)?),
                    Rule::type_alias => {
                        let alias = parser::parse_type_alias(inner)?;
                        type_aliases.insert(alias.name.clone(), alias);
                    }
                    _ => {}
                }
            }
            Rule::EOI => {}
            _ => {}
        }
    }

    let model_names: HashSet<String> = raw_models.iter().map(|m| m.name.clone()).collect();

    let enum_names: HashSet<String> = enums.iter().map(|e| e.name.clone()).collect();

    let tables = resolver::resolve_models(raw_models, &model_names, &enum_names, &type_aliases)?;

    let views = resolver::resolve_views(raw_views, &enum_names, &type_aliases)?;

    let mut schema = Schema::new(database);

    schema.tables = tables;
    schema.enums = enums;
    schema.views = views;

    Ok(schema)
}

#[cfg(test)]
mod tests;
