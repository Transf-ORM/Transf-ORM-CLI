//! Snapshot tests of the diagnostics rendered for invalid Prisma schemas.
//!
//! Review snapshot changes with `cargo insta review`.

use super::parse_schema;
use crate::importer::ImportErrorKind;

/// Parses `input`, expecting a failure, and renders the error as the CLI would.
fn render(input: &str) -> String {
    let err = parse_schema(input).expect_err("schema should be rejected");
    assert!(err.location().is_some(), "diagnostic must carry a location");
    err.with_path("schema.prisma").to_string()
}

#[test]
fn missing_field_type() {
    let input = r#"datasource db {
  provider = "postgresql"
}

model User {
  id    Int @id
  email
}
"#;
    insta::assert_snapshot!(render(input));
}

#[test]
fn unclosed_model_block() {
    let input = r#"model User {
  id    Int    @id
  email String @unique
"#;
    insta::assert_snapshot!(render(input));
}

#[test]
fn unknown_top_level_keyword() {
    let input = r#"model User {
  id Int @id
}

modle Post {
  id Int @id
}
"#;
    insta::assert_snapshot!(render(input));
}

#[test]
fn out_of_range_integer() {
    let input = r#"model Counter {
  id    Int @id
  value Int @default(99999999999999999999)
}
"#;
    let err = parse_schema(input).unwrap_err();
    assert_eq!(err.kind(), ImportErrorKind::Schema);
    insta::assert_snapshot!(render(input));
}
