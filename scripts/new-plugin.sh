#!/usr/bin/env bash
# Stamp a new stave plugin project — the whole canonical shape,
# gate-passing on arrival: declarations, an honest-passthrough
# engine, a house-styled editor, and the full colocated test suite
# (every framework check from one line, a face test, a starter property),
# in the shape of the framework's examples/tilt.
#
# Usage:
#   scripts/new-plugin.sh <name>       # lowercase, hyphens ok: "fuzz", "tape-echo"
#
# Produces plugins/<name>/ (crate stave-<name>-vst3, bundle
# Stave<Name>), registers the workspace member, and prints the next
# steps. VST3 class ids are freshly generated and collision-checked
# against every existing plugin — ids are forever, so they are minted
# exactly once, here (and added to conformance/tests/identity.rs).
#
# The plugin's INFO vendor fields come from the environment (set them
# for your own plugins; the defaults are the stave author's):
#   STAVE_BUNDLE_VENDOR  reverse-DNS id prefix   (com.ryanbahan.stave)
#   STAVE_VENDOR_NAME    vendor display name     (Ryan Bahan)
#   STAVE_VENDOR_URL     vendor / project URL    (https://github.com/stave-dsp/stave)

set -euo pipefail

NAME="${1:?usage: new-plugin.sh <name> (lowercase, hyphens ok)}"

if [[ ! "$NAME" =~ ^[a-z][a-z0-9-]*$ ]]; then
  echo "error: name must be lowercase alphanumeric with hyphens: '$NAME'" >&2
  exit 1
fi

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
source "${REPO_ROOT}/scripts/lib/bundle-common.sh"
VENDOR_NAME="${STAVE_VENDOR_NAME:-Ryan Bahan}"
VENDOR_URL="${STAVE_VENDOR_URL:-https://github.com/stave-dsp/stave}"
DIR="${REPO_ROOT}/plugins/${NAME}"
CRATE="stave-${NAME}-vst3"
UNDERSCORED="${CRATE//-/_}"

if [[ -d "$DIR" ]]; then
  echo "error: ${DIR} already exists" >&2
  exit 1
fi

# Display name: "tape-echo" -> "Tape Echo"; struct name: "TapeEcho".
DISPLAY=$(echo "$NAME" | awk -F- '{for(i=1;i<=NF;i++){printf "%s%s%s", (i>1?" ":""), toupper(substr($i,1,1)), substr($i,2)}}')
STRUCT="${DISPLAY// /}"
DISPLAY_UPPER=$(echo "$DISPLAY" | tr '[:lower:]' '[:upper:]')

# Mint a VST3 class id: four random u32 words, collision-checked against
# every class id already in the tree (ids are FOREVER).
mint_cid() {
  while :; do
    local words
    words=$(od -An -tx4 -N16 /dev/urandom | tr -s ' ' '\n' | grep -v '^$' \
      | awk '{printf "0x%s%s", toupper($0), (NR<4?", ":"")}' \
      | sed 's/0X/0x/g')
    local first=${words%%,*}
    if ! grep -rq "$first" "${REPO_ROOT}/plugins" 2>/dev/null; then
      echo "[$words]"
      return
    fi
  done
}
COMPONENT_CID=$(mint_cid)
CONTROLLER_CID=$(mint_cid)
BUNDLE_VENDOR="${STAVE_BUNDLE_VENDOR:-com.ryanbahan.stave}"
PLUGIN_ID="${BUNDLE_VENDOR}.${NAME//-/}.vst3"

mkdir -p "${DIR}/src" "${DIR}/tests"

# ---------------------------------------------------------------- Cargo.toml
cat > "${DIR}/Cargo.toml" <<EOF
[package]
name = "${CRATE}"
version = "0.0.1"
edition.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "Stave ${DISPLAY} (scaffold: honest passthrough)."
publish = false

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
stave.workspace = true

[dev-dependencies]
stave-test.workspace = true

[lints]
workspace = true
EOF

# ------------------------------------------------------------------- lib.rs
cat > "${DIR}/src/lib.rs" <<EOF
//! Stave ${DISPLAY} — params, processing and face, top to bottom (the
//! shape of the framework's examples/tilt). Split into params.rs,
//! editor.rs and engine/ the day a second concept arrives.
//!
//! STATUS: scaffold — an honest passthrough (output trim + bypass).

use stave::prelude::*;

#[derive(Params)]
pub struct ${STRUCT}Params {
    pub output: FloatParam,
    pub bypass: BoolParam,
}

impl Default for ${STRUCT}Params {
    fn default() -> Self {
        // Ids are FOREVER: pick once, never reuse or renumber.
        ${STRUCT}Params {
            output: FloatParam::new(ParamId(1), "Output", Range::linear(-24.0, 24.0))
                .unit(Unit::Db)
                .smooth(Smooth::ms(5.0)),
            bypass: BoolParam::bypass(ParamId(2), "Bypass"),
        }
    }
}

pub struct ${STRUCT} {
    params: Arc<${STRUCT}Params>,
    output: Smoother,
}

impl Plugin for ${STRUCT} {
    const INFO: Info = Info {
        id: "${PLUGIN_ID}",
        name: "Stave ${DISPLAY}",
        vendor: "${VENDOR_NAME}",
        url: "${VENDOR_URL}",
        features: &[Feature::AudioEffect, Feature::Stereo],
        ..Info::EFFECT
    };
    type Params = ${STRUCT}Params;

    fn new(params: &Arc<${STRUCT}Params>, _telemetry: &Telemetry) -> Self {
        ${STRUCT} { params: params.clone(), output: Smoother::new(&params.output) }
    }

    fn prepare(&mut self, env: &Environment) {
        self.output.prepare(env);
    }

    fn process(&mut self, block: &mut Block<'_>) {
        let mut output = self.output.ramp(block, &self.params.output);
        block.lanes::<2>().run(|frame| {
            let db = output.next();
            // Unity is bit-exact: no multiply at 0 dB.
            let gain = if db == 0.0 { 1.0 } else { db_to_amp(db) };
            for lane in 0..frame.lanes() {
                frame.write(lane, frame.input(lane) * gain);
            }
        });
    }

    fn face(p: &${STRUCT}Params) -> Option<Face> {
        let theme = Theme::graphite();
        let header = row(vec![
            Title::new("${DISPLAY_UPPER}").tagline("scaffold · honest passthrough").node(),
            Footswitch::bypass(p.bypass.bind()).compact().node(),
        ])
        .gap(theme.space(4))
        .align(Align::Start);
        let body = column(vec![header, Plate::titled("OUTPUT").around(Knob::new(p.output.bind()).huge().centered().node())])
            .gap(theme.space(3))
            .align(Align::Stretch);
        Some(Face::new(theme, Panel::default().around(body)))
    }
}

// The class ids are FOREVER (saved sessions key on them): add them to
// conformance/tests/identity.rs.
stave::export!(${STRUCT}, vst3_class_ids: {
    component: ${COMPONENT_CID},
    controller: ${CONTROLLER_CID},
});
EOF

# -------------------------------------------------------------------- tests
cat > "${DIR}/tests/checks.rs" <<EOF
//! Every check ${STRUCT} owes (validation, no allocation, perf, the
//! face's laws, host behaviour), from one line.

stave_test::plugin_checks!(${UNDERSCORED}::${STRUCT}, {
    perf: vec![PerfCase { name: "DEFAULT", set: &[] }],
});
EOF

cat > "${DIR}/tests/editor.rs" <<EOF
//! The face, as a person uses it.

use stave_test::face::{dy, FaceDriver};
use ${UNDERSCORED}::${STRUCT};

#[test]
fn dragging_the_output_knob_up_raises_it() {
    let mut ui = FaceDriver::of::<${STRUCT}>();
    ui.press("output").move_to(dy(-55.0)).release();
    assert!(ui.value("output") > 0.0);
    ui.assert_bracketed();
}
EOF

cat > "${DIR}/tests/properties.rs" <<EOF
//! ${STRUCT}'s OWN properties — the tests only this project can write.
//! The shared checks prove the plugin is SANE; only a property proves
//! it is RIGHT (the framework's examples/tilt/tests/properties.rs is the
//! cautionary tale: a backwards knob passed every shared check). Write
//! the first property BEFORE the DSP it constrains.
//!
//! The starter property pins the scaffold's one promise — unity
//! passthrough is BIT-exact. Replace it the day the engine grows a voice.

use stave_test::PluginHarness;
use ${UNDERSCORED}::${STRUCT};

const BLOCK: usize = 256;

#[test]
fn unity_passthrough_is_bit_exact() {
    let mut h = PluginHarness::<${STRUCT}>::builder().sample_rate(48_000.0).block_size(BLOCK).build();
    let input: Vec<f32> = (0..BLOCK).map(|i| (i as f32 * 0.13).sin() * 0.5).collect();
    for _ in 0..8 {
        for ch in h.inputs_mut() {
            ch.copy_from_slice(&input);
        }
        h.process(&[]);
        for (i, (x, y)) in input.iter().zip(&h.outputs()[0]).enumerate() {
            assert!(x.to_bits() == y.to_bits(), "sample {i}: {x} vs {y}");
        }
    }
}
EOF

# --------------------------------------------------- workspace registration
python3 - "$REPO_ROOT" "$NAME" <<'PYEOF'
import sys
root, name = sys.argv[1], sys.argv[2]
p = f"{root}/Cargo.toml"
s = open(p).read()
member = f'    "plugins/{name}",\n'
if member in s:
    print("workspace: already registered")
else:
    i = s.index("members = [")
    j = s.index("]", i)
    s = s[:j] + member + s[j:]
    open(p, "w").write(s)
    print(f"workspace: registered plugins/{name}")
PYEOF

echo ""
echo "stamped: plugins/${NAME}  (crate ${CRATE}, bundle Stave${STRUCT}.vst3)"
echo ""
echo "next steps:"
echo "  cargo test -p ${CRATE}                     # every check, green on arrival"
echo "  bash scripts/install.sh ${CRATE} release   # build + bundle + validate + install"
echo "  stave-bench build/Stave${STRUCT}.vst3 --describe          # tools: see README, Building"
echo "  stave-play  build/Stave${STRUCT}.vst3 --screenshot /tmp/${NAME}.png"
echo ""
echo "then add the class ids to conformance/tests/identity.rs, read stave's examples/tilt,"
echo "and write your first property in tests/properties.rs before the DSP it constrains."
