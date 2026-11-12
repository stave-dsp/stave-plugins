#!/usr/bin/env bash
# G6 grep bar: engines publish observations ONLY through declared Tap
# handles — no name-string publishing at engine call sites. (The
# framework's own view-binding and layering bars live in its repo.)
set -euo pipefail
cd "$(dirname "$0")/.."
hits=$(grep -rn "probe!\|probe_f64\|probe_named" plugins/*/src --include="*.rs" || true)
if [[ -n "$hits" ]]; then
  echo "G6 VIOLATION: name-keyed tap publishing in an engine (declare a Tap):"; echo "$hits"; exit 1
fi
