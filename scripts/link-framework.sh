#!/usr/bin/env bash
# Co-edit the framework: build the plugins against a local checkout of
# github.com/stave-dsp/stave instead of the pinned release tag.
#
#   scripts/link-framework.sh            # link ../stave
#   scripts/link-framework.sh PATH       # link another checkout
#   scripts/link-framework.sh --off      # back to the pinned tag
#   scripts/link-framework.sh --status
#
# Linking appends a marked [patch] block to Cargo.toml (and rewrites
# Cargo.lock's framework entries to paths). Never commit it linked: CI
# fails on the block. To ship framework changes: stave's scripts/release.sh,
# then scripts/bump-framework.sh here.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
URL="https://github.com/stave-dsp/stave"
BEGIN="# >>> link-framework (local only — never commit)"
END="# <<< link-framework"

linked() { grep -qF "$BEGIN" Cargo.toml; }

unlink_block() {
  python3 - "$BEGIN" "$END" <<'PY'
import sys
begin, end = sys.argv[1], sys.argv[2]
s = open("Cargo.toml").read()
i = s.index(begin); j = s.index(end, i) + len(end)
open("Cargo.toml", "w").write(s[:i].rstrip("\n") + "\n" + s[j:].lstrip("\n"))
PY
}

case "${1:-}" in
  --status)
    if linked; then
      echo "linked: $(grep -m1 'path = ' Cargo.toml | sed -E 's/.*path = "([^"]+)\/crates.*/\1/')"
    else
      echo "pinned: $(grep -m1 "git = \"$URL\"" Cargo.toml | sed -E 's/.*tag = "([^"]+)".*/\1/')"
    fi
    ;;
  --off)
    linked || { echo "already pinned"; exit 0; }
    unlink_block
    cargo metadata -q --format-version 1 >/dev/null
    echo "pinned: framework back on the release tag"
    ;;
  *)
    FW="$(cd "${1:-../stave}" && pwd)"
    [[ -f "$FW/crates/stave-core/Cargo.toml" ]] || { echo "not a stave checkout: $FW" >&2; exit 1; }
    linked && unlink_block
    crates=$(grep -E "^stave-[a-z0-9-]+ = \{ git = \"$URL\"" Cargo.toml | cut -d' ' -f1)
    {
      echo ""
      echo "$BEGIN"
      echo "[patch.\"$URL\"]"
      for c in $crates; do echo "$c = { path = \"$FW/crates/$c\" }"; done
      echo "$END"
    } >> Cargo.toml
    cargo metadata -q --format-version 1 >/dev/null
    echo "linked: $FW ($(echo "$crates" | wc -w | tr -d ' ') crates)"
    ;;
esac
