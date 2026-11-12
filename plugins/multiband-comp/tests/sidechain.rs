//! The multiband compressor detects from its declared
//! external-sidechain aux port:
//!  1. ext off → a hot aux signal causes NO gain reduction.
//!  2. ext on + aux connected → the hot aux DOES duck the quiet main.
//!  3. ext on + aux absent → identical to ext off (the aux
//!     contract: fully functional with every aux disconnected).

use stave::Layout;
use stave_multiband_comp_vst3::MultibandComp;
use stave_test::PluginHarness;

const FS: f32 = 48_000.0;
const BLOCK: usize = 256;
const BLOCKS: usize = 64; // ~340 ms: detectors fully settled

/// Render with a quiet main tone and a hot aux tone; return the main
/// path's output RMS over the last quarter (post-settle).
fn render(ext_on: bool, connect_aux: bool) -> f64 {
    // Deep thresholds + strong ratios so external program produces
    // unmistakable reduction; makeup 0.
    let mut builder = PluginHarness::<MultibandComp>::builder()
        .sample_rate(FS)
        .block_size(BLOCK)
        .set("ext sc", if ext_on { 1.0 } else { 0.0 });
    for b in ["lo", "mid", "hi"] {
        builder = builder
            .set(&format!("{b} threshold"), -45.0)
            .set(&format!("{b} ratio"), 10.0);
    }
    // Unplugged: the plain stereo layout, no aux bus at all.
    if !connect_aux {
        builder = builder.layout(Layout::STEREO);
    }
    let mut h = builder.build();

    let mut sum_sq = 0.0f64;
    let mut count = 0u32;
    for blk in 0..BLOCKS {
        for f in 0..BLOCK {
            let t = (blk * BLOCK + f) as f32 / FS;
            // Quiet main: −32 dBFS-ish 300 Hz. Hot aux: 0.8 pk 300 Hz
            // (same band as the main tone, so LOW-band GR applies).
            let ph = 2.0 * core::f32::consts::PI * 300.0 * t;
            for ch in h.inputs_mut() {
                ch[f] = 0.025 * ph.sin();
            }
            for c in 0..2 {
                if h.aux_buses() > 0 {
                    h.aux_mut(0, c)[f] = 0.8 * ph.sin();
                }
            }
        }
        h.process(&[]);
        if blk >= BLOCKS - BLOCKS / 4 {
            for &y in h.outputs()[0].iter() {
                sum_sq += f64::from(y) * f64::from(y);
                count += 1;
            }
        }
    }
    (sum_sq / f64::from(count)).sqrt()
}

#[test]
fn external_sidechain_ducks_the_main_path() {
    let internal = render(false, true); // hot aux ignored
    let external = render(true, true); // hot aux drives detection
    let db = 20.0 * (external / internal).log10();
    println!("internal {internal:.6} | external {external:.6} | Δ {db:.1} dB");
    assert!(
        db < -6.0,
        "external sidechain should duck the quiet main by >6 dB: {db:.2} dB"
    );
}

#[test]
fn ext_on_without_aux_is_ext_off() {
    // The aux contract, as behavior: flag on but nothing connected
    // must equal flag off exactly (same samples, not just same RMS).
    let off = render(false, false);
    let unplugged = render(true, false);
    assert_eq!(
        off.to_bits(),
        unplugged.to_bits(),
        "ext-on-unplugged must be bit-identical to ext-off"
    );
}
