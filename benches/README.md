# Benchmark protocol — `convert`

This folder measures how `transf-orm convert` behaves as the schema grows: **execution
time** and **peak resident memory (max RSS)**, on Prisma schemas of increasing size.

```bash
bash benches/run.sh                 # 10, 100, 1,000 and 5,000 models
bash benches/run.sh 10 100          # only some sizes
RUNS=50 bash benches/run.sh         # more runs
```

Requirements: Rust toolchain, `python3`, [`hyperfine`](https://github.com/sharkdp/hyperfine)
and GNU `time` (`/usr/bin/time`), on Linux.

## What is measured

| | |
|---|---|
| Command | `target/release/transf-orm convert --from prisma <schema>` |
| Build | `cargo build --release`, rebuilt by the script before measuring |
| Output | stdout discarded (`--output=null`), so terminal rendering is not measured |
| Time | wall-clock time of the whole process, measured by hyperfine (`--shell=none`, so no shell startup) |
| Memory | peak resident set size, from GNU `time -f %M` |

`--from prisma` is used so that the command never stops on the ORM-detection confirmation
prompt.

The time includes process startup, reading the file, the Prisma importer (pest parsing +
AST → pivot), and serializing the canonical JSON pivot. It is the time a user waits for.

## Input schemas

`benches/gen_schema.py <n>` generates a valid Prisma schema with `n` models. The output
is **deterministic**, so two measurements made months apart run on identical input.

Each model has 12 scalar fields covering the importer's common paths:

- `@id`, `@unique`, `@default` (autoincrement, uuid, now, literals, enum values) and `@updatedAt`;
- `@map` and `@db.*` hints;
- optional fields, arrays, `Json`, `Decimal`, and a field typed by one of 3 shared enums.

Each model also has a relation to the previous model (with `onDelete`) and its
back-relation, a `@@map` and two `@@index`. One model in ten adds a compound
`@@unique`.

| Models | Size | Role |
|---:|---:|---|
| 10 | ~8 KB | close to a small real project (`test.prisma` is 9.4 KB); mostly measures startup |
| 100 | ~78 KB | medium project |
| 1,000 | ~780 KB | large monorepo schema |
| 5,000 | ~3.9 MB | stress size, shows the growth trend |

The schemas are written to `benches/schemas/`, which git ignores: only the generator is
versioned.

## Runs

- **Time**: hyperfine, `--warmup 3 --runs 30` per size. The warmup runs fill the disk
  cache, so every measured run reads the file from memory. The minimum required by the
  protocol is 20 runs.
- **Memory**: 5 separate runs per size; the highest max RSS is kept.

The script checks that `convert` succeeds on every schema before measuring it: a failing
conversion would otherwise be timed as a fast run.

## Conditions

Before running:

- plug in the machine (no battery power-saving);
- close heavy applications (browser, IDE indexing, builds, VMs);
- do not use the machine during the run (about 2 minutes).

The script records the machine automatically in `machine.md`:

- CPU, cores and threads, RAM, OS, kernel, CPU governor, power source;
- `rustc` and `hyperfine` versions, release binary size;
- commit, and whether `crates/` had uncommitted changes.

## Results

Each run writes a folder `benches/results/<YYYY-MM-DD>-<commit>/`, **which is committed**:

| File | Content |
|---|---|
| `summary.md` | one row per size: schema size, mean ± σ, min, max, number of runs, max RSS |
| `machine.md` | the measurement machine and conditions |
| `hyperfine-<n>.json` | raw hyperfine export, with every individual run time |
| `hyperfine-<n>.md` | hyperfine's Markdown table |

### Reading the results

- Look at **how time and memory grow between sizes**, not only at absolute values. If ×10
  models costs about ×10 time, the growth is linear. Anything clearly above that points
  to a quadratic step to investigate.
- At 10 models the time is dominated by process startup. Differences of less than a
  millisecond there say nothing about the parser.
- A σ larger than ~10 % of the mean means the machine was not quiet: rerun.

### Comparing two measurements

Only compare results from the **same machine** (same `machine.md`), with the same sizes.
To compare two commits directly, run hyperfine on both binaries in one session:

```bash
hyperfine --warmup 3 --runs 30 --shell=none --output=null \
  './old/transf-orm convert --from prisma benches/schemas/models-1000.prisma' \
  './target/release/transf-orm convert --from prisma benches/schemas/models-1000.prisma'
```

A change counts as a regression when the mean grows by more than 2 σ on two consecutive
measurements.

## Known limits

- **Synthetic schemas**: they are regular and cover the common Prisma attributes, but not
  views, multi-schema, `Unsupported` types or deep nesting.
  - Deeply nested attribute values currently crash the importer (#210), so they cannot be
    measured yet.
- **One machine**: absolute numbers depend on the CPU. Only the trend and comparisons on
  the same machine are meaningful.
- **No CI benchmark**: shared CI runners are too noisy for absolute values.
- **Not covered**: only `convert` (Prisma → pivot JSON) is measured, since it is the only
  available command. Exporters and the server mode will need their own protocol.
