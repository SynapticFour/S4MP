#!/usr/bin/env bash
# Mirror GitHub CI cargo gates for S4MP.
set -euo pipefail
# macOS 27: ld-1267 cannot read arm64e.x1 in the CLT SDK. See macos-sdk.sh.
# shellcheck disable=SC1091
source "$(git rev-parse --show-toplevel)/scripts/hooks/macos-sdk.sh"
use_linkable_macos_sdk
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

echo "ci-check: cargo fmt --check"
cargo fmt --all -- --check

echo "ci-check: cargo clippy --locked"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "ci-check: cargo test --locked"
cargo test --locked --workspace

echo "ci-check: cargo doc --locked"
RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --no-deps

echo "ci-check: dependency tiers"
bash "$ROOT/scripts/check-tiers.sh"

if command -v cargo-deny >/dev/null 2>&1; then
  echo "ci-check: cargo deny"
  cargo deny check
else
  echo "ci-check: cargo-deny not installed (CI runs it); skipping locally"
fi

echo "ci-check: OK"
