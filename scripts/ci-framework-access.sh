#!/usr/bin/env bash
# Give this repo's CI read-only access to the framework while
# github.com/stave-dsp/stave is private.
#
#   scripts/ci-framework-access.sh            # create (or rotate) the access
#   scripts/ci-framework-access.sh --revoke   # remove it (once stave is public)
#   scripts/ci-framework-access.sh --status
#
# A fresh ed25519 key pair is generated in a private temp directory: the
# public half becomes a READ-ONLY deploy key on the framework repo, the
# private half goes straight from that file into the Actions secret
# STAVE_DEPLOY_KEY over stdin (never echoed, never in the repo, never in
# shell history). The temp directory is wiped on exit, success or not.
# Re-running rotates: the previous key is deleted before the new one is
# added. Needs `gh` logged in with admin on both repos.
set -euo pipefail

FRAMEWORK="${STAVE_FRAMEWORK_REPO:-stave-dsp/stave}"
PLUGINS="${STAVE_PLUGINS_REPO:-stave-dsp/stave-plugins}"
SECRET=STAVE_DEPLOY_KEY
TITLE="${PLUGINS} CI (read-only)"

key_ids() {
  gh api "repos/${FRAMEWORK}/keys" --jq ".[] | select(.title == \"${TITLE}\") | .id"
}

revoke() {
  for id in $(key_ids); do
    gh api -X DELETE "repos/${FRAMEWORK}/keys/${id}" >/dev/null
    echo "deleted deploy key ${id} from ${FRAMEWORK}"
  done
  if gh secret list --repo "$PLUGINS" | grep -q "^${SECRET}\b"; then
    gh secret delete "$SECRET" --repo "$PLUGINS"
    echo "deleted secret ${SECRET} from ${PLUGINS}"
  fi
}

case "${1:-}" in
  --status)
    ids=$(key_ids)
    echo "deploy key on ${FRAMEWORK}: ${ids:-none}"
    gh secret list --repo "$PLUGINS" | grep "^${SECRET}\b" || echo "secret ${SECRET} on ${PLUGINS}: none"
    ;;
  --revoke)
    revoke
    ;;
  "")
    umask 077
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT
    ssh-keygen -q -t ed25519 -N '' -C "$TITLE" -f "$tmp/key"
    revoke
    gh api -X POST "repos/${FRAMEWORK}/keys" \
      -f title="$TITLE" -f key="$(cat "$tmp/key.pub")" -F read_only=true >/dev/null
    gh secret set "$SECRET" --repo "$PLUGINS" < "$tmp/key"
    echo "CI access ready: read-only deploy key on ${FRAMEWORK}, secret ${SECRET} on ${PLUGINS}"
    echo "re-run CI: gh workflow run ci -R ${PLUGINS}"
    ;;
  *)
    echo "usage: $0 [--status|--revoke]" >&2; exit 2
    ;;
esac
