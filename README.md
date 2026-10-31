# Transf-ORM-CLI

A powerful command-line tool for transforming and managing ORM schemas with ease.

[![Security](https://github.com/Transf-ORM/Transf-ORM-CLI/actions/workflows/security.yml/badge.svg)](https://github.com/Transf-ORM/Transf-ORM-CLI/actions/workflows/security.yml)
[![Build](https://github.com/Transf-ORM/Transf-ORM-CLI/actions/workflows/build.yml/badge.svg)](https://github.com/Transf-ORM/Transf-ORM-CLI/actions/workflows/build.yml)

## Features

**Legend:** ✅ Available · 🚧 In progress · 📋 Planned

| Status | Feature | Description |
|--------|---------|-------------|
| ✅ | **Pivot IR** | Universal intermediate representation (tables, columns, relations, enums, views, behaviors…) serialized as canonical JSON |
| ✅ | **Prisma import** | Full `schema.prisma` parsing into the pivot (attributes, relations, `@map`/`@@map`, views, createdAt/updatedAt/softDelete/versioning detection) |
| ✅ | **ORM auto-detection** | Source ORM detected from the file extension, or forced with `--from` |
| ✅ | **Execution modes** | A single binary with two modes: CLI (default, `--cli`) and server (`--serveur`) |
| 🚧 | **Global flags** | `--verbose` (progress steps) and `--debug <LEVEL>` (internal logs) work; `--perf`, `--tts` and `--daltonien` are accepted but have no effect yet |
| 📋 | **Interactive CLI** | Interactive prompt that shows the ORM detected in the current directory |
| 📋 | **HTTP server** | REST API served by `--serveur` (currently only logs its startup) |
| 🚧 | **Drizzle import** | Drizzle schema parsing into the pivot |
| 📋 | **ORM export** | Pivot → Prisma, Drizzle, then TypeORM and other SQL ORMs |
| 📋 | **Code generation** | TypeScript types, Zod validators, SQL DDL from the pivot |
| 📋 | **Codebase migration (codemods)** | AST-based rewrite of ORM call sites when switching ORM (e.g. `prisma.user.findMany()` → `db.select().from(users)`) |
| 📋 | **Schema versioning** | Diff, merge, history and rollback |
| 📋 | **Multi-database support** | PostgreSQL, MySQL and SQLite are modeled in the pivot; dialect-specific export is not implemented yet. NoSQL is out of scope |

Adding a new ORM only requires a new entry in [`crates/core/src/registry.rs`](crates/core/src/registry.rs).

## Prerequisites

- Rust 1.94 or later (`rustup update stable`)
- Cargo
- Docker, only to run the server's database locally

## Installation

### From npm

```bash
npm install -g @transf-orm/cli
```

### From Source

```bash
git clone https://github.com/Transf-ORM/Transf-ORM-CLI.git
cd Transf-ORM-CLI
cargo build --release
```

The binary will be available at `./target/release/transf-orm`

### Add to PATH

```bash
cp ./target/release/transf-orm /usr/local/bin/
```

## Usage

### Basic Command

```bash
transf-orm [OPTIONS] [COMMAND]
```

### Modes

The binary runs in one of two modes:

| Mode | How to run it | Status |
|------|---------------|--------|
| CLI | `transf-orm`, `transf-orm --cli` or `transf-orm <command>` | ✅ commands · 📋 interactive mode (without a command, prints a placeholder message) |
| Server | `transf-orm --serveur [--port <PORT>]` | 📋 only logs its startup for now |

`--cli` and `--serveur` cannot be combined, and `--serveur` cannot be used with a command.

### Examples

```bash
# Display help (modes, global flags and commands)
transf-orm --help

# Display the version
transf-orm --version

# Convert a Prisma schema to the pivot IR (canonical JSON on stdout)
transf-orm convert schema.prisma

# Save the pivot to a file
transf-orm convert schema.prisma > pivot.json

# Force the source ORM (skips auto-detection)
transf-orm convert --from prisma path/to/schema

# Global flags go before or after the command
transf-orm --verbose convert schema.prisma

# Show each step on stderr — stdout stays pure JSON, so piping still works
transf-orm --verbose convert schema.prisma > pivot.json

# Show internal logs up to the given level (on stderr)
transf-orm --debug debug convert schema.prisma > pivot.json

# Start the server mode on a given port
transf-orm --serveur --port 8080
```

### Planned Commands

| Status | Command | Description |
|--------|---------|-------------|
| ✅ | `convert <path>` | Import a schema into the pivot IR |
| 📋 | `convert --to <orm>` | Convert a schema to another ORM |
| 📋 | `generate` | Generate TypeScript types, Zod validators, SQL DDL |
| 📋 | `diff` / `merge` | Compare and merge schema versions |
| 📋 | `validate` | Validate a schema |
| 📋 | `init` | Initialize a project configuration |
| 📋 | `watch` | Re-run conversion on file changes |
| 📋 | `history` | Browse schema history |

## Configuration

There is no configuration file yet — everything is passed through CLI flags.

### Global Flags

Global flags apply to both modes and can be placed anywhere on the command line.

| Flag | Description | Status |
|------|-------------|--------|
| `--cli` | Run the CLI mode (default when no mode is given) | ✅ |
| `--serveur` | Run the server mode | ✅ |
| `--debug <LEVEL>` | Print internal logs on stderr up to `LEVEL`: `error`, `warn`, `info`, `debug` or `trace` (case-insensitive) | ✅ |
| `--verbose` | Print each step of the command on stderr, prefixed with `[verbose]` | ✅ |
| `--perf` | Report performance metrics | 🚧 parsed, no effect yet |
| `--tts` | Read CLI feedback aloud (text-to-speech) | 🚧 parsed, no effect yet |
| `--daltonien` | Use a colour-blind-friendly palette | 🚧 parsed, no effect yet |
| `-h`, `--help` | Print help | ✅ |
| `-V`, `--version` | Print the version | ✅ |

`--verbose` and `--debug` only write to stderr, so the output on stdout (e.g. the JSON pivot) is the same with or without them. Without either flag, nothing extra is printed.

An unknown flag, an invalid `--debug` level or an invalid combination (`--cli` with `--serveur`, `--serveur` with a command, `--port` without `--serveur`) prints an error and exits with code `2`.

### Server Flags

| Flag | Description | Default |
|------|-------------|---------|
| `--port <PORT>` | Port to listen on (`0` = any free port). Requires `--serveur` | `3000` |

📋 `--ip <ADDRESS>` and `--host <HOST>` are planned.

### Command Options

| Option | Command | Description | Accepted values |
|--------|---------|-------------|-----------------|
| `--from <ORM>` | `convert` | Force the source ORM instead of detecting it from the file extension | `prisma` |

📋 A project configuration file generated by `transf-orm init` is planned.

## Project Structure

```
.
├── crates/
│   ├── core/            # Business logic
│   │   └── src/
│   │       ├── pivot/       # Pivot IR (schema, tables, relations, views…)
│   │       ├── importer/    # ORM schema parsers → pivot (Prisma)
│   │       ├── exporter/    # Pivot → ORM schemas (planned)
│   │       ├── config.rs    # GlobalConfig: global flags shared by both modes
│   │       └── registry.rs  # Known ORMs, detection, importer/exporter factories
│   ├── cli/             # CLI mode: commands
│   ├── server/          # Server mode: server flags, HTTP API (planned)
│   │   └── src/db/          # Persistence (SeaORM): entities, migrations
│   └── transf-orm/      # Binary: global flags parsing + --cli / --serveur dispatch
│       └── tests/           # Integration tests on the real binary
├── bin/
│   └── transf-orm       # Shell wrapper used by the npm package
├── Cargo.toml           # Workspace manifest
├── docker-compose.yml   # Development PostgreSQL for the server mode
├── package.json         # npm package (@transf-orm/cli)
└── .github/
    ├── workflows/
    └── ISSUE_TEMPLATE/
```

## Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style

- Format with `cargo fmt`
- Run `cargo clippy --all-targets --all-features -- -D warnings`
- Write tests for new features
- Update documentation as needed

### Reporting Issues

- Use the [bug report template](.github/ISSUE_TEMPLATE/bug_report.md) for bug reports
- Use the [feature request template](.github/ISSUE_TEMPLATE/feature_request.md) for feature suggestions
- Be as descriptive as possible

## Testing

Run tests with:

```bash
cargo test
```

Run tests with verbose output:

```bash
cargo test -- --nocapture
```

The server's database tests (`db_smoke`) need PostgreSQL. Start it with Docker; without it, they are skipped locally (and fail in CI):

```bash
docker compose up -d                        # PostgreSQL on localhost:5442
cargo test -p transf-orm-server db_smoke
docker compose down -v                      # stop it and delete its data
```

The port can be changed with `TRANSF_ORM_DB_PORT`; the tests then read the URL from `DATABASE_URL`.

## Security

We take security seriously. Please see our [security policy](SECURITY.md) for reporting vulnerabilities.

Run security audits with:

```bash
cargo audit
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- Check the [wiki](https://github.com/Transf-ORM/Transf-ORM-CLI/wiki) for detailed guides
- Open an issue for questions or issues
- Report bugs using the [bug report template](.github/ISSUE_TEMPLATE/bug_report.md)

## Acknowledgments

Thanks to all contributors and the open-source community for making this project possible.
