#!/usr/bin/env bash
# Usage: check.sh <lesson.md>
# Compiles every ```rust block in the lesson (clippy pedantic). Dependencies come
# from the lesson's ```toml block (bare `name = ...` lines). Prints ok/FAILED per block.
set -euo pipefail

lesson=${1:?usage: check.sh <lesson.md>}
[[ -f $lesson ]] || { echo "no such file: $lesson" >&2; exit 1; }

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# Split rust blocks into $work/blockN.rs and collect dependency lines.
awk -v d="$work" '
  /^```rust/ { n++; mode = "rust"; next }
  /^```toml/ { mode = "toml"; next }
  /^```/     { mode = ""; next }
  mode == "rust" { print > (d "/block" n ".rs") }
  mode == "toml" && $0 !~ /^#/ && NF { print > (d "/deps.toml") }
' "$lesson"

shopt -s nullglob
blocks=("$work"/block*.rs)
((${#blocks[@]})) || { echo "no rust blocks in $lesson" >&2; exit 1; }

status=0
for src in "${blocks[@]}"; do
  name=$(basename "$src" .rs)
  proj="$work/proj-$name"
  mkdir -p "$proj/src"
  {
    printf '[package]\nname = "%s"\nversion = "0.0.0"\nedition = "2021"\n\n[dependencies]\n' "$name"
    [[ -f $work/deps.toml ]] && cat "$work/deps.toml"
  } > "$proj/Cargo.toml"
  cp "$src" "$proj/src/main.rs"
  if out=$(cd "$proj" && cargo clippy -q -- -W clippy::pedantic 2>&1); then
    if [[ -n $out ]]; then echo "$name: warnings"; echo "$out"; status=1; else echo "$name: ok"; fi
  else
    echo "$name: FAILED"; echo "$out"; status=1
  fi
done
exit $status
