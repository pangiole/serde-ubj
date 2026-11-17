#!/usr/bin/env bash

set -euo pipefail
readonly script_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

no_std=''
strict=''
alloc=''

# Minimum line coverage (percent) enforced by `cargo llvm-cov`; the build fails
# if any pipeline drops below it.
coverage_min=80

pipeline='std'
cargo_features=''

usage() {
  cat <<'EOF'
Usage:
  ./build.sh                      # standard environments
  ./build.sh --no-std --alloc     # no standard, yet heap allocation environemnts
  ./build.sh --no-std --strict    # no standard, strictly no heap allocation
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-std) no_std=1 ;;
    --strict) strict=1 ;;
    --alloc)  alloc=1 ;;
    -h|--help) usage; exit 0 ;;
    *) printf 'error: unknown argument: %s\n' "$1" >&2; usage >&2; exit 1 ;;
  esac
  shift
done

if [[ -n "$no_std" ]]; then
  if [[ -n "$strict" && -z "$alloc" ]]; then
    pipeline='no_std (strict)'
    cargo_features='--no-default-features'
  elif [[ -n "$alloc" && -z "$strict" ]]; then
    pipeline='no_std (alloc)'
    cargo_features='--no-default-features --features alloc'
  else
    printf 'error: use exactly one of --strict or --alloc with --no-std\n' >&2
    usage >&2
    exit 1
  fi
elif [[ -n "$strict" || -n "$alloc" ]]; then
  printf 'error: --strict and --alloc require --no-std\n' >&2
  usage >&2
  exit 1
fi

function step {
  echo " "
  echo " "
  echo "$1  =============================="
  echo " "
}

step "BUILDING: $pipeline"
echo "cargo features: ${cargo_features:-<default features>}"

step "CLEANING"
cargo clean

step "LINTING"
cargo clippy $cargo_features

step "TESTING"
cargo llvm-cov $cargo_features --fail-under-lines "$coverage_min"

step "BUILDING DOCS"
cargo test $cargo_features --doc -- --show-output
cargo doc $cargo_features --verbose --no-deps

exit 0

