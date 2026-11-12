#!/usr/bin/env bash
# Move the plugins to a new framework release.
#
#   scripts/bump-framework.sh v0.2.0
#
# Rewrites the tag on every framework dependency in Cargo.toml, refreshes
# Cargo.lock to that tag's commit, then runs the gates. Commit the result
# yourself once it is green. Refuses to run while linked
# (scripts/link-framework.sh) — a linked build would hide the tag.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
URL="https://github.com/stave-dsp/stave"
TAG="${1:?usage: $0 <tag, e.g. v0.2.0>}"

if grep -qF ">>> link-framework" Cargo.toml; then
  echo "linked to a local framework: run scripts/link-framework.sh --off first" >&2; exit 1
fi
git ls-remote --exit-code --tags "$URL" "refs/tags/${TAG}" >/dev/null \
  || { echo "no tag ${TAG} on ${URL}" >&2; exit 1; }

current=$(grep -m1 "git = \"$URL\"" Cargo.toml | sed -E 's/.*(tag|rev) = "([^"]+)".*/\2/')
sed -i.bak -E "s#(git = \"${URL//./\\.}\", )(tag|rev) = \"[^\"]+\"#\1tag = \"${TAG}\"#" Cargo.toml && rm Cargo.toml.bak
crates=$(grep -E "^stave(-[a-z0-9-]+)? = \{ git = \"$URL\"" Cargo.toml | cut -d' ' -f1)
# shellcheck disable=SC2086
cargo update $(printf -- '-p %s ' $crates)
echo "framework: ${current} -> ${TAG}"
bash scripts/gates.sh
