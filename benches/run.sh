#!/usr/bin/env bash
# Benchmark `transf-orm convert` on synthetic Prisma schemas of increasing size.
#
# Protocol: see benches/README.md.
# Output:   benches/results/<YYYY-MM-DD>-<commit>/
#
# Usage:    bash benches/run.sh [sizes...]      (default: 10 100 1000 5000)
# Env:      RUNS (default 30), WARMUP (default 3), RSS_RUNS (default 5)

set -euo pipefail

cd "$(dirname "$0")/.."

SIZES=("$@")
[ ${#SIZES[@]} -eq 0 ] && SIZES=(10 100 1000 5000)
RUNS=${RUNS:-30}
WARMUP=${WARMUP:-3}
RSS_RUNS=${RSS_RUNS:-5}
BIN=target/release/transf-orm
GNU_TIME=/usr/bin/time

for tool in hyperfine python3 "$GNU_TIME"; do
    command -v "$tool" >/dev/null || { echo "error: $tool is required" >&2; exit 1; }
done
"$GNU_TIME" --version 2>&1 | grep -q GNU || { echo "error: $GNU_TIME must be GNU time" >&2; exit 1; }

commit=$(git rev-parse --short HEAD)
dirty=""
git diff --quiet HEAD -- crates Cargo.toml Cargo.lock || dirty=" (uncommitted changes in crates/)"
out="benches/results/$(date +%F)-$commit"
mkdir -p "$out" benches/schemas

echo "==> Building release binary"
cargo build --release --quiet

# ── Machine ─────────────────────────────────────────────────────────────────
{
    echo "# Measurement machine"
    echo
    echo "| | |"
    echo "|---|---|"
    echo "| Date | $(date '+%F %T %Z') |"
    echo "| Commit | \`$commit\`$dirty |"
    echo "| CPU | $(lscpu | sed -n 's/^Model name:[[:space:]]*//p' | head -1) |"
    echo "| Cores / threads | $(lscpu -p=CORE | grep -v '^#' | sort -u | wc -l) / $(nproc) |"
    echo "| RAM | $(free -h | awk '/^Mem:/ {print $2}') |"
    echo "| OS | $( (. /etc/os-release && echo "$PRETTY_NAME") 2>/dev/null || uname -s) |"
    echo "| Kernel | $(uname -r) |"
    echo "| CPU governor | $(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo n/a) |"
    echo "| Power | $(cat /sys/class/power_supply/A*/online 2>/dev/null | head -1 | sed 's/1/AC/;s/0/battery/' || echo n/a) |"
    echo "| rustc | $(rustc --version) |"
    echo "| hyperfine | $(hyperfine --version) |"
    echo "| Binary size | $(stat -c%s "$BIN") bytes |"
    echo "| Runs / warmup | $RUNS / $WARMUP (RSS: $RSS_RUNS runs, max kept) |"
} > "$out/machine.md"

# ── Measurements ────────────────────────────────────────────────────────────
summary_rows=()
for n in "${SIZES[@]}"; do
    schema="benches/schemas/models-$n.prisma"
    python3 benches/gen_schema.py "$n" > "$schema"
    size_kb=$(awk -v b="$(stat -c%s "$schema")" 'BEGIN {printf "%.1f", b / 1024}')

    # The run is only meaningful if the conversion succeeds.
    "$BIN" convert --from prisma "$schema" > /dev/null

    echo "==> $n models ($size_kb KB)"
    hyperfine --warmup "$WARMUP" --runs "$RUNS" --shell=none --output=null \
        --export-json "$out/hyperfine-$n.json" \
        --export-markdown "$out/hyperfine-$n.md" \
        --command-name "convert $n models" \
        "$BIN convert --from prisma $schema"

    max_rss=0
    for _ in $(seq "$RSS_RUNS"); do
        rss=$("$GNU_TIME" -f %M "$BIN" convert --from prisma "$schema" 2>&1 >/dev/null | tail -1)
        [ "$rss" -gt "$max_rss" ] && max_rss=$rss
    done

    row=$(python3 - "$out/hyperfine-$n.json" <<'EOF'
import json, sys
r = json.load(open(sys.argv[1]))["results"][0]
ms = lambda s: f"{s * 1000:.2f}"
print(f"{ms(r['mean'])} ± {ms(r['stddev'])} | {ms(r['min'])} | {ms(r['max'])} | {len(r['times'])}")
EOF
)
    summary_rows+=("| $n | $size_kb | $row | $(awk -v k="$max_rss" 'BEGIN {printf "%.1f", k / 1024}') |")
done

{
    echo "# convert benchmark — $(date +%F), commit \`$commit\`"
    echo
    echo "Machine: see [machine.md](machine.md). Protocol: see [benches/README.md](../../README.md)."
    echo
    echo "| Models | Schema (KB) | Mean ± σ (ms) | Min (ms) | Max (ms) | Runs | Max RSS (MB) |"
    echo "|---:|---:|---:|---:|---:|---:|---:|"
    printf '%s\n' "${summary_rows[@]}"
} > "$out/summary.md"

echo
cat "$out/summary.md"
echo
echo "Results written to $out/"
