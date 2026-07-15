#!/usr/bin/env bash
set -euo pipefail
git config core.hooksPath .githooks
echo "Git hooks path set to .githooks/ — pre-commit checks are now active."
