# convert benchmark — 2026-11-12, commit `0de65ad`

Machine: see [machine.md](machine.md). Protocol: see [benches/README.md](../../README.md).

| Models | Schema (KB) | Mean ± σ (ms) | Min (ms) | Max (ms) | Runs | Max RSS (MB) |
|---:|---:|---:|---:|---:|---:|---:|
| 10 | 7.9 | 4.17 ± 0.34 | 3.44 | 4.76 | 30 | 3.9 |
| 100 | 78.0 | 21.86 ± 1.87 | 18.80 | 25.22 | 30 | 10.2 |
| 1000 | 781.9 | 223.54 ± 7.95 | 210.78 | 250.01 | 30 | 75.4 |
| 5000 | 3926.2 | 1527.13 ± 37.37 | 1478.73 | 1645.86 | 30 | 363.5 |

## Observations

- **10 → 100 models (×10)**: time ×5.2. At 10 models, the 4 ms are mostly process
  startup.
- **100 → 1,000 models (×10)**: time ×10.2, so the growth is linear.
- **1,000 → 5,000 models (×5)**: time ×6.8. The growth becomes slightly superlinear
  (~1.4× worse than linear). To investigate before relying on very large schemas.
- **Memory**: max RSS grows linearly, at about 73 KB per model, which is **~93× the schema
  size** (363 MB for a 3.9 MB file). Memory, not time, is the first limit for very large
  schemas. Likely causes are the pest parse tree, the AST and the pivot all held in memory
  together, and the full JSON string built before printing.
- σ stays under 10 % of the mean for every size (largest: 8.6 % at 100 models).
