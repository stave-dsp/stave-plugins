# stave-plugins

Audio plugins built on [stave](https://github.com/stave-dsp/stave), a
from-scratch Rust plugin framework with hand-written CLAP and VST3
bindings.

Every plugin ships as native macOS `.vst3` + `.clap` bundles from one
cdylib and is held to the same gauntlet: stave's validation suite, the
Steinberg VST3 validator and clap-validator
(`scripts/validate-formats.sh` runs both locally).

## Plugins (`plugins/`)

**Stable** — feature-complete, with their own property suites:

| plugin | category | what it is |
|---|---|---|
| Stave Forge | distortion | amp-topology saturation: two clipping stages with interstage memory, slew grit, iron hysteresis |
| Stave Multiband | dynamics | 3-band compressor with Linkwitz-Riley crossover and external sidechain |
| Stave Smear | resonator | Sympathetic Garden — self-tuning resonator field; smear blooms across pitches |
| Stave Eddy | delay | a feedback delay whose read head breathes: four feedback laws, an in-loop diffusion field |

**In development** — usable, still moving:

| plugin | category | what it is |
|---|---|---|
| Stave Saga | reverse / reverb | two time machines in series (reverse delays and reverbs) on a 32 kHz clock you can pull; see plugins/saga/docs/design.md |
| Stave Heirloom | reverse / lo-fi | Saga's reverse machines played back off a worn record: platter wobble and a generation-loss wear stage |
| Stave Ribbon | delay | an impossible tape echo: a tape loop modelled in tape coordinates, and an ERASE knob you can turn down |
| Stave Vault | reverb | a synthesized room (live image-source early field) into a nonlinear Hadamard FDN tank |
| Stave Engram | reverb | a reverb with associative memory: attention over remembered grains of your playing shapes the tail |
| Stave Revenant | generative | an echo-state network that learns your playing online and keeps playing when you stop |
| Stave Aeolian | resonator | a sympathetic string harp: twelve plucked-string voices that ring with what you play |
| Stave Anvil | amp | a multiband metalcore amp head: parallel triode cascades, push-pull power, per-band saturation |
| Stave Halo | amp | a high-headroom clean amp whose attack can stay cleaner than its sustain |
| Stave Tolex | cabinet | a guitar cabinet built, not sampled: speaker motor, cone, box and air as a physical model |
| Stave Dynamo | synth | a physical-modelling synth built from machinery that never existed |
| Stave Opal | synth | a 4-operator FM synth |
| Stave Chime | synth | a polyphonic sine/saw synth with a modulation matrix |

`conformance/` holds the cross-plugin suites: plugin engines measured
against the framework's alias, zipper, invariance and preset contracts.

## Building

The plugins build against a sibling checkout of the framework:

```sh
git clone https://github.com/stave-dsp/stave
git clone https://github.com/stave-dsp/stave-plugins
cd stave-plugins

# Build + bundle + validate + install one plugin (fresh inodes —
# never cp -R over a loaded bundle):
bash scripts/install.sh stave-eddy-vst3 release

# Open it in stave's host:
cargo run --release --manifest-path ../stave/Cargo.toml -p stave-play -- \
  "$PWD/build/StaveEddy.vst3"
```

`scripts/bundle-all.sh` builds and installs every plugin.

## The gates

```sh
scripts/gates.sh            # tree rules, every test in parallel, doctests
scripts/gates.sh affected   # only the packages the change can reach
```

Tests run under [cargo-nextest](https://nexte.st). CI holds rustc and
clippy at zero warnings and runs both format validators on every plugin.
`docs/host-matrix.md` is the manual DAW checklist.

## Adding a plugin

```sh
bash scripts/new-plugin.sh myplugin
```

That stamps a complete, gate-passing plugin under `plugins/` — then read
stave's `docs/writing-a-plugin.md` with its `examples/tilt` beside it.

## License

MIT, see [LICENSE](LICENSE).
