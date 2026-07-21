# Transf-ORM-CLI Documentation

Technical documentation for the actual, current implementation of Transf-ORM-CLI. For the
long-term product vision (business model, SaaS roadmap, PoC criteria), see
[`project.md`](../project.md) at the repo root — this `docs/` folder only describes what
exists in the code today.

## Contents

| Document | What it covers |
|---|---|
| [`architecture.md`](architecture.md) | The 6-layer design, what's implemented vs. planned, module dependency graph |
| [`pivot-ir.md`](pivot-ir.md) | The Pivot IR in depth — every type, the DB/ORM layer split, relations, metadata, serialization, with Mermaid diagrams |
| [`importers.md`](importers.md) | The `Importer` trait and the Prisma importer's PEG → AST → resolve pipeline; how to add a new importer |
| [`cli.md`](cli.md) | The `transf-orm convert` command, ORM detection, and the `registry` module |

## Implementation status

Transf-ORM-CLI has three long-term goals — schema conversion, code generation, and
codebase migration (see the root [`CLAUDE.md`](../CLAUDE.md) for the full picture). Today:

- ✅ **Pivot IR** (`src/pivot/`) — fully implemented: all types, serde, canonical JSON, tests.
- ✅ **Prisma importer** (`src/importer/prisma/`) — complete PEG grammar, full attribute
  coverage, 30+ tests.
- ✅ **CLI** (`src/main.rs`) — one command, `convert`, reads a schema file and prints the
  canonical JSON pivot.
- 🚧 **Exporters** (`src/exporter/`) — only the `Exporter` trait exists, no implementation yet.
- 🚧 **Drizzle importer** — not started.
- 🚧 **Codemods, versioning/diff/merge** — not started.

See [`architecture.md`](architecture.md) for how these pieces fit together.

## Quick start

```bash
cargo build --release
./target/release/transf-orm-cli convert path/to/schema.prisma
# prints the canonical JSON pivot to stdout
```
