#!/usr/bin/env python3
"""Split the workspace's packages into CI shards that take about equally
long: each shard compiles and tests only its own packages, so both the
build and the tests divide (a nextest `--partition` divides only the
tests; every shard would still build everything).

    scripts/ci-shard.py 2/3      # print `-p a -p b ...` for shard 2 of 3
    scripts/ci-shard.py --show 3 # print every shard with its weight

The weights are per OS, read from $RUNNER_OS (default macOS).

Every workspace package lands in exactly one shard, so the shards of one
OS together run every test. A package missing from WEIGHTS (a new
plugin) gets DEFAULT_WEIGHT; refresh the table when the balance drifts.
"""

import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# What a package costs a CI runner, per OS (macOS, Linux, Windows): its
# build (CPU-seconds of its own crates in `cargo test --no-run
# --timings`; dependencies come from the cache) plus its tests
# (core-seconds: nextest durations, times the cores a test holds, see
# .config/nextest.toml). Measured 2026-11-12 on framework v0.4.0 on
# macos-latest, ubuntu-latest and windows-latest. Tolex is mostly
# build (seven test binaries each instantiate rustfft); Engram mostly
# tests that hold every core; Conformance links most plugins' libraries.
WEIGHTS = {
    "stave-engram-vst3": (181, 176, 230),
    "stave-tolex-vst3": (150, 174, 341),
    "stave-vault-vst3": (72, 119, 174),
    "stave-anvil-vst3": (61, 52, 100),
    "stave-plugins-conformance": (59, 43, 70),
    "stave-smear-vst3": (40, 38, 70),
    "stave-halo-vst3": (26, 21, 35),
    "stave-forge-vst3": (26, 21, 33),
    "stave-eddy-vst3": (24, 22, 36),
    "stave-dynamo-vst3": (24, 18, 29),
    "stave-heirloom-vst3": (21, 17, 27),
    "stave-multiband-comp-vst3": (21, 17, 28),
    "stave-opal-vst3": (20, 14, 24),
    "stave-revenant-vst3": (20, 16, 26),
    "stave-ribbon-vst3": (18, 14, 23),
    "stave-aeolian-vst3": (18, 13, 22),
    "stave-saga-vst3": (18, 14, 22),
    "stave-chime-vst3": (16, 10, 17),
    "stave-profile-multiband": (1, 1, 1),
}
DEFAULT_WEIGHT = (25, 20, 35)
OS_COLUMN = {"macOS": 0, "Linux": 1, "Windows": 2}  # $RUNNER_OS
OS = os.environ.get("RUNNER_OS", "macOS")


def weight(name):
    return WEIGHTS.get(name, DEFAULT_WEIGHT)[OS_COLUMN[OS]]


def packages():
    out = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps", "--locked"],
        cwd=ROOT, check=True, capture_output=True, text=True,
    ).stdout
    return sorted(p["name"] for p in json.loads(out)["packages"])


def shards(count):
    """Longest-first greedy: each package to the lightest shard so far."""
    bins = [[0, []] for _ in range(count)]
    for name in sorted(packages(), key=lambda n: (-weight(n), n)):
        lightest = min(bins, key=lambda b: b[0])
        lightest[0] += weight(name)
        lightest[1].append(name)
    assigned = [n for _, names in bins for n in names]
    assert sorted(assigned) == packages() and len(set(assigned)) == len(assigned)
    return bins


def main(argv):
    if len(argv) == 2 and argv[0] == "--show":
        for i, (weight, names) in enumerate(shards(int(argv[1])), 1):
            print(f"{i}: {weight:4d}  {' '.join(names)}")
        return 0
    if len(argv) == 1 and "/" in argv[0]:
        index, count = (int(x) for x in argv[0].split("/"))
        if not 1 <= index <= count:
            raise SystemExit(f"shard {index} of {count}?")
        print(" ".join(f"-p {n}" for n in shards(count)[index - 1][1]))
        return 0
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
