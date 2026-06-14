#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WHITE_PAPER="$REPO_ROOT/WHITE_PAPER.md"

if [[ ! -f "$WHITE_PAPER" ]]; then
  echo "Missing WHITE_PAPER.md at $WHITE_PAPER" >&2
  exit 1
fi

grep -q '\*\*Author:\*\* Mohamad Supardi Abdul' "$WHITE_PAPER"
grep -q '\*\*Official Contact:\*\* `mymsastudio@gmail.com`' "$WHITE_PAPER"
grep -q '^# Logicodex Language White Paper' "$WHITE_PAPER"

if grep -q 'Author: Manus AI\|# Logica Language White Paper' "$WHITE_PAPER"; then
  echo "Obsolete primary Logica or Manus AI attribution found in WHITE_PAPER.md" >&2
  exit 1
fi

if ! grep -qE '^\[[0-9]+\]:' "$WHITE_PAPER"; then
  echo "Reference definitions are missing from WHITE_PAPER.md" >&2
  exit 1
fi

printf 'Validated consolidated Logicodex white paper: %s\n' "$WHITE_PAPER"
wc -l -w -c "$WHITE_PAPER"
