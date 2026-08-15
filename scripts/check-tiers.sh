#!/usr/bin/env bash
# Enforce S4MP crate dependency tiers (outer may depend on inner only).
# Reads only the [dependencies] table of each Cargo.toml (not comments, not
# [dev-dependencies]). Parked crates are skipped.
# Tier map: docs/engineering/ENGINEERING_STANDARDS.md §3.3
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

tier_of() {
  case "$1" in
    s4-core) echo 0 ;;
    s4-storage|s4-events|s4-plugin|s4-project) echo 1 ;;
    s4-parser|s4-graph|s4-knowledge|s4-requirements|s4-metrics|s4-analysis) echo 2 ;;
    s4-verification|s4-certification|s4-llm) echo 3 ;;
    s4-cli) echo 4 ;;
    s4-planner|s4-api|s4-ui) echo parked ;;
    *) echo "" ;;
  esac
}

extract_deps() {
  awk '
    /^\[dependencies\]/ {d=1; next}
    /^\[/ {d=0}
    d && $0 ~ /^[[:space:]]*#/ {next}
    d {
      line = $0
      while (match(line, /s4-[a-z0-9-]+/)) {
        print substr(line, RSTART, RLENGTH)
        line = substr(line, RSTART + RLENGTH)
      }
    }
  ' "$1"
}

errors=0
for crate_dir in crates/s4-*; do
  crate="$(basename "$crate_dir")"
  cargo_toml="$crate_dir/Cargo.toml"
  [ -f "$cargo_toml" ] || continue
  my_tier="$(tier_of "$crate")"
  if [ "$my_tier" = "parked" ]; then
    continue
  fi
  if [ -z "$my_tier" ]; then
    echo "error: unknown crate tier for $crate" >&2
    errors=$((errors + 1))
    continue
  fi
  deps="$(extract_deps "$cargo_toml" | sort -u | grep -v "^${crate}$" || true)"
  for dep in $deps; do
    dep_tier="$(tier_of "$dep")"
    [ -n "$dep_tier" ] || continue
    if [ "$dep_tier" = "parked" ]; then
      echo "error: $crate depends on parked crate $dep" >&2
      errors=$((errors + 1))
      continue
    fi
    if [ "$dep_tier" -gt "$my_tier" ]; then
      echo "error: $crate (tier $my_tier) depends on $dep (tier $dep_tier) — upward dependency" >&2
      errors=$((errors + 1))
    fi
  done
done

if [ "$errors" -gt 0 ]; then
  echo "tier check failed with $errors error(s)" >&2
  exit 1
fi
echo "tier check ok"
