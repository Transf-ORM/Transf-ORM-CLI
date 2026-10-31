use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

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

/// Runs the binary in `dir` with `input` on stdin, as typed in the prompt.
fn run_interactive(args: &[&str], dir: &Path, input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_transf-orm"))
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to run the transf-orm binary");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn fixtures() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures"))
}

#[test]
fn no_argument_runs_interactive_cli() {
    let output = run_interactive(&[], fixtures(), "/exit\n");
    assert!(output.status.success());
    assert!(stdout(&output).contains("interactive mode"));
}

#[test]
fn cli_flag_runs_interactive_cli() {
    let output = run_interactive(&["--cli"], fixtures(), "/exit\n");
    assert!(output.status.success());
    assert!(stdout(&output).contains("interactive mode"));
}

#[test]
fn interactive_prints_detected_orm() {
    let output = run_interactive(&[], fixtures(), "");
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains("Detected ORM: Prisma (verbose.prisma)"));
}

#[test]
fn interactive_without_schema_prints_no_orm() {
    let dir = std::env::temp_dir().join(format!("transf-orm-empty-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let output = run_interactive(&[], &dir, "");
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(output.status.success());
    assert!(stdout(&output).contains("No ORM detected"));
}

#[test]
fn interactive_help_lists_commands() {
    let output = run_interactive(&[], fixtures(), "/help\n/exit\n");
    assert!(output.status.success());
    let out = stdout(&output);
    for expected in ["/help", "/exit", "convert"] {
        assert!(out.contains(expected), "/help is missing {expected}: {out}");
    }
}

#[test]
fn interactive_exit_stops_reading() {
    let output = run_interactive(&[], fixtures(), "/exit\n/nope\n");
    assert!(output.status.success());
    assert_eq!(stderr(&output), "");
}

#[test]
fn interactive_errors_keep_the_session_open() {
    let output = run_interactive(
        &[],
        fixtures(),
        "nope\n/nope\nconvert missing.prisma\nconvert verbose.prisma\n",
    );
    assert!(output.status.success());
    let errors = stderr(&output);
    assert!(errors.contains("error: unknown command 'nope'"), "{errors}");
    assert!(
        errors.contains("error: unknown command '/nope'"),
        "{errors}"
    );
    assert!(
        errors.contains("error: could not read missing.prisma"),
        "{errors}"
    );
    // The command after the errors still ran.
    assert!(stdout(&output).contains("\"pivotVersion\""));
}

/// Runs the binary in a new empty directory holding `schemas` (each an
/// empty file), then removes it.
fn run_in_dir_with(name: &str, schemas: &[&str], args: &[&str]) -> Output {
    let dir = std::env::temp_dir().join(format!("transf-orm-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for schema in schemas {
        std::fs::copy(fixtures().join("verbose.prisma"), dir.join(schema)).unwrap();
    }
    let output = Command::new(env!("CARGO_BIN_EXE_transf-orm"))
        .args(args)
        .current_dir(&dir)
        .output()
        .expect("failed to run the transf-orm binary");
    std::fs::remove_dir_all(&dir).unwrap();
    output
}

#[test]
fn convert_without_path_uses_the_detected_schema() {
    let output = run_in_dir_with("one-schema", &["schema.prisma"], &["convert"]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(stderr(&output), "Using detected schema: schema.prisma\n");
    assert!(stdout(&output).contains("\"pivotVersion\""));
}

#[test]
fn convert_without_path_needs_exactly_one_schema() {
    let none = run_in_dir_with("no-schema", &[], &["convert"]);
    assert_eq!(none.status.code(), Some(1));
    assert!(
        stderr(&none).contains("no schema file found"),
        "{}",
        stderr(&none)
    );

    let two = run_in_dir_with("two-schemas", &["a.prisma", "b.prisma"], &["convert"]);
    assert_eq!(two.status.code(), Some(1));
    let errors = stderr(&two);
    assert!(errors.contains("2 schema files found"), "{errors}");
    assert!(
        errors.contains("a.prisma") && errors.contains("b.prisma"),
        "{errors}"
    );
}

#[test]
fn interactive_convert_without_path_uses_the_detected_schema() {
    let output = run_interactive(&[], fixtures(), "convert\n");
    assert!(output.status.success());
    assert!(stderr(&output).contains("Using detected schema: verbose.prisma"));
    assert!(stdout(&output).contains("\"pivotVersion\""));
}

#[test]
fn command_error_exits_with_1() {
    let output = run_in_crate(&["convert", "missing.prisma"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).starts_with("error: could not read missing.prisma"));
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
