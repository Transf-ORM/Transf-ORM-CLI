//! Integration tests of the CLI mode, driving the real `transf-orm` binary.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

const VALID_SCHEMA: &str = "tests/fixtures/verbose.prisma";
const BROKEN_SCHEMA: &str = "tests/fixtures/broken.prisma";

/// The `transf-orm` binary, run from the crate directory so fixture paths
/// stay relative.
fn transf_orm() -> Command {
    let mut cmd = Command::cargo_bin("transf-orm").expect("binary should be built");
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"));
    cmd
}

#[test]
fn convert_prints_pivot_json() {
    let output = transf_orm()
        .args(["convert", VALID_SCHEMA])
        .assert()
        .success()
        .stderr("")
        .get_output()
        .stdout
        .clone();

    let pivot: Value = serde_json::from_slice(&output).expect("stdout should be valid JSON");
    assert_eq!(pivot["database"], "postgreSql");
    assert_eq!(pivot["tables"][0]["name"], "User");
    assert_eq!(pivot["enums"][0]["name"], "Role");
}

#[test]
fn convert_with_from_skips_detection() {
    transf_orm()
        .args(["convert", "--from", "prisma", VALID_SCHEMA])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("{"));
}

#[test]
fn convert_invalid_schema_fails_with_parse_error() {
    transf_orm()
        .args(["convert", "--from", "prisma", BROKEN_SCHEMA])
        .assert()
        .code(1)
        .stdout("")
        .stderr(predicate::str::contains("parse error"));
}

#[test]
fn convert_missing_file_fails() {
    transf_orm()
        .args([
            "convert",
            "--from",
            "prisma",
            "tests/fixtures/does-not-exist.prisma",
        ])
        .assert()
        .code(1)
        .stdout("")
        .stderr(predicate::str::contains(
            "error reading tests/fixtures/does-not-exist.prisma",
        ));
}

#[test]
fn convert_unknown_orm_fails() {
    transf_orm()
        .args(["convert", "--from", "doctrine", VALID_SCHEMA])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "unknown ORM 'doctrine'. Valid options: prisma",
        ));
}

#[test]
fn convert_without_path_is_a_usage_error() {
    transf_orm()
        .arg("convert")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("<PATH>"));
}
