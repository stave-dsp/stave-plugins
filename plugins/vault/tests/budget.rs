//! THE PERF RATCHET — hard per-machine ceilings, generous enough to
//! ignore machine noise (median ~830 ns/frame across sessions; a single
//! 489 ns reading after the tape purge was a MACHINE-STATE FLUKE that
//! briefly mis-lowered these ceilings — never ratchet on one run),
//! tight enough to catch the regression class the field caught by hand
//! (per-tap cubic scattering, 1263 ns/frame). Lower a ceiling when the
//! median legitimately drops. The perf REPORT runs in `checks.rs`.

use stave_test::PluginHarness;
use stave_vault_vst3::Vault;

/// THE ROOM BUDGET.
#[test]
#[cfg_attr(miri, ignore)]
fn the_machine_stays_inside_its_budget() {
    budget_for(&[], 1_000.0);
}

/// THE PLATE BUDGET — the topology arc's CPU thesis, pinned: a
/// plate runs NO geometry stages (no image source, no midfield,
/// no modes), so it must come in far under the room.
#[test]
#[cfg_attr(miri, ignore)]
fn the_plate_stays_inside_its_budget() {
    budget_for(&[("topology", 1.0)], 600.0);
}

/// THE MEMORY BUDGET: 126 resonators and nothing else.
#[test]
#[cfg_attr(miri, ignore)]
fn the_memory_stays_inside_its_budget() {
    budget_for(&[("topology", 2.0)], 700.0);
}

/// THE CHAMBER BUDGET.
#[test]
#[cfg_attr(miri, ignore)]
fn the_chamber_stays_inside_its_budget() {
    budget_for(&[("topology", 3.0)], 600.0);
}

/// THE VOID BUDGET.
#[test]
#[cfg_attr(miri, ignore)]
fn the_void_stays_inside_its_budget() {
    budget_for(&[("topology", 4.0)], 700.0);
}

fn budget_for(set: &[(&str, f64)], release_ceiling: f64) {
    const SAMPLE_RATE: f32 = 48_000.0;
    const BLOCK: usize = 256;
    let mut h = set
        .iter()
        .fold(PluginHarness::<Vault>::builder().sample_rate(SAMPLE_RATE).block_size(BLOCK), |b, (n, v)| b.set(n, *v))
        .build();
    let mut lcg: u32 = 7;
    let mut run = |blocks: usize| -> f64 {
        let t0 = std::time::Instant::now();
        for _ in 0..blocks {
            let mut noise = [0.0f32; BLOCK];
            for x in noise.iter_mut() {
                lcg = lcg.wrapping_mul(1664525).wrapping_add(1013904223);
                *x = ((lcg >> 8) as f32 / (1u32 << 23) as f32 - 1.0) * 0.3;
            }
            for ch in h.inputs_mut() {
                ch.copy_from_slice(&noise);
            }
            h.process(&[]);
        }
        t0.elapsed().as_secs_f64() / (blocks * BLOCK) as f64 * 1e9
    };
    run(100); // warm
    let mut samples: Vec<f64> = (0..5).map(|_| run(200)).collect();
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = samples[2];
    println!("  MEDIAN {median:.0} ns/frame ({:.1}% of the 48k budget)", median / 20_833.0 * 100.0);
    // Debug builds run ~10-20x slower — the ceiling scales.
    let ceiling = if cfg!(debug_assertions) { 22_000.0 } else { release_ceiling };
    assert!(median < ceiling, "perf regression: {median:.0} ns/frame (ceiling {ceiling:.0})");
}
