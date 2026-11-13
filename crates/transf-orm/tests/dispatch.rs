use std::process::{Command, Output};

const VERBOSE_SCHEMA: &str = "tests/fixtures/verbose.prisma";

const TEST_SCHEMA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../test.prisma");

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_transf-orm"))
        .args(args)
        .output()
        .expect("failed to run the transf-orm binary")
}

/// Runs the binary from the crate directory, so fixture paths stay relative
/// and the output does not depend on where the repository is checked out.
fn run_in_crate(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_transf-orm"))
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("failed to run the transf-orm binary")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn version_prints_crate_version() {
    let output = run(&["--version"]);
    assert!(output.status.success());
    assert_eq!(
        stdout(&output).trim(),
        format!("transf-orm {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn help_lists_modes_and_commands() {
    let output = run(&["--help"]);
    assert!(output.status.success());
    let help = stdout(&output);
    for expected in [
        "--cli",
        "--serveur",
        "--debug",
        "--verbose",
        "--perf",
        "--tts",
        "--daltonien",
        "convert",
    ] {
        assert!(help.contains(expected), "help is missing {expected}");
    }
}

#[test]
fn no_argument_runs_interactive_cli() {
    let output = run(&[]);
    assert!(output.status.success());
    assert!(stdout(&output).contains("Interactive mode"));
}

#[test]
fn cli_flag_runs_interactive_cli() {
    let output = run(&["--cli"]);
    assert!(output.status.success());
    assert!(stdout(&output).contains("Interactive mode"));
}

#[test]
fn cli_mode_runs_commands() {
    let output = run(&["--cli", "--verbose", "convert", TEST_SCHEMA]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains("\"pivotVersion\""));
}

#[test]
fn serveur_flag_starts_server() {
    let output = run(&["--serveur", "--port", "0", "--debug", "info"]);
    assert!(output.status.success());
    assert!(stderr(&output).contains("Starting transf-orm server on port 0"));
}

#[test]
fn unknown_flag_exits_with_2() {
    let output = run(&["--nope"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("--nope"));
}

#[test]
fn invalid_debug_level_exits_with_2() {
    assert_eq!(run(&["--debug", "bogus"]).status.code(), Some(2));
}

#[test]
fn conflicting_modes_exit_with_2() {
    assert_eq!(run(&["--cli", "--serveur"]).status.code(), Some(2));
    assert_eq!(
        run(&["--serveur", "convert", TEST_SCHEMA]).status.code(),
        Some(2)
    );
    assert_eq!(run(&["--port", "0"]).status.code(), Some(2));
}

#[test]
fn verbose_output_snapshot() {
    let output = run_in_crate(&["--verbose", "convert", VERBOSE_SCHEMA]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        stderr(&output),
        "\
[verbose] Detecting ORM format for tests/fixtures/verbose.prisma
[verbose] Source ORM: Prisma
[verbose] Reading tests/fixtures/verbose.prisma
[verbose] Importing schema with the Prisma importer
[verbose] Imported 1 tables, 0 views, 1 enums
[verbose] Writing canonical JSON pivot to stdout
"
    );
}

#[test]
fn verbose_with_from_reports_forced_orm() {
    let output = run_in_crate(&["--verbose", "convert", "--from", "prisma", VERBOSE_SCHEMA]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stderr(&output).starts_with("[verbose] Using ORM forced by --from: prisma\n"));
}

#[test]
fn verbose_keeps_stdout_unchanged() {
    let plain = run_in_crate(&["convert", VERBOSE_SCHEMA]);
    let verbose = run_in_crate(&["--verbose", "convert", VERBOSE_SCHEMA]);
    assert_eq!(stdout(&plain), stdout(&verbose));
    assert!(stdout(&verbose).trim_start().starts_with('{'));
}

#[test]
fn no_flag_prints_nothing_on_stderr() {
    let output = run_in_crate(&["convert", VERBOSE_SCHEMA]);
    assert!(output.status.success());
    assert_eq!(stderr(&output), "");
}

#[test]
fn debug_enables_internal_logs() {
    let output = run_in_crate(&["--debug", "debug", "convert", VERBOSE_SCHEMA]);
    assert!(output.status.success(), "{}", stderr(&output));
    let logs = stderr(&output);
    assert!(logs.contains("DEBUG"), "{logs}");
    assert!(logs.contains("parsed Prisma blocks"), "{logs}");
    assert!(!logs.contains("TRACE"), "{logs}");
    assert!(
        !logs.contains('\x1b'),
        "logs must not contain ANSI codes when piped"
    );
}

#[test]
fn debug_trace_shows_every_block() {
    let output = run_in_crate(&["--debug", "trace", "convert", VERBOSE_SCHEMA]);
    let logs = stderr(&output);
    assert!(logs.contains("TRACE"), "{logs}");
    assert!(logs.contains("model=User"), "{logs}");
}

#[test]
fn debug_error_hides_lower_levels() {
    let output = run_in_crate(&["--debug", "error", "convert", VERBOSE_SCHEMA]);
    assert!(output.status.success());
    assert_eq!(stderr(&output), "");
}
