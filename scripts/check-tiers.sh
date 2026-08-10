#!/usr/bin/env bash
# Enforce S4MP crate dependency tiers (outer may depend on inner only).
# Tier map: docs/engineering/ENGINEERING_STANDARDS.md §3.3
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

tier_of() {
  case "$1" in
    s4-core) echo 0 ;;
    s4-storage|s4-events|s4-plugin|s4-project) echo 1 ;;
    s4-parser|s4-graph|s4-knowledge|s4-requirements|s4-metrics|s4-analysis) echo 2 ;;
    s4-planner|s4-verification|s4-certification|s4-llm) echo 3 ;;
    s4-api|s4-cli|s4-ui) echo 4 ;;
    *) echo "" ;;
  esac
}

# Known exceptions (tracked debt). Format: "dependent:dependency"
# s4-project embeds LanguageId from s4-parser; relocate LanguageId to s4-core in Phase 3.
is_allowed_exception() {
  case "$1:$2" in
    s4-project:s4-parser) return 0 ;;
    *) return 1 ;;
  esac
}

errors=0
for crate_dir in crates/s4-*; do
  crate="$(basename "$crate_dir")"
  cargo_toml="$crate_dir/Cargo.toml"
  [ -f "$cargo_toml" ] || continue
  my_tier="$(tier_of "$crate")"
  if [ -z "$my_tier" ]; then
    echo "error: unknown crate tier for $crate" >&2
    errors=$((errors + 1))
    continue
  fi
  deps="$(grep -oE 's4-[a-z0-9-]+' "$cargo_toml" | sort -u | grep -v "^${crate}$" || true)"
  for dep in $deps; do
    dep_tier="$(tier_of "$dep")"
    [ -n "$dep_tier" ] || continue
    if [ "$dep_tier" -gt "$my_tier" ]; then
      if is_allowed_exception "$crate" "$dep"; then
        echo "warn: allowed exception $crate → $dep (tier $my_tier → $dep_tier)"
        continue
      fi
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
