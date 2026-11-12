//! The audition bench (ignored by default): every factory preset over
//! a plucked A-minor phrase, HOLD latched through the second half of the
//! tail, written to build/listen/heirloom-<preset>.wav, with the dry and
//! wet-only levels printed (the wet-balance record). Run with
//! `cargo test --release -p stave-heirloom-vst3 --test listen -- --ignored --nocapture`.

use stave::prelude::*;
use stave_heirloom_vst3::{Heirloom, HeirloomParams};
use stave_test::PluginHarness;

const FS: f32 = 48_000.0;
const BLOCK: usize = 256;

fn rms(x: &[f32]) -> f32 {
    stave_measure::level::rms(x) as f32
}

/// Karplus–Strong plucks: A minor, up and back, then ring out.
fn phrase() -> Vec<f32> {
    let mut out = vec![0.0f32; (FS * 12.0) as usize];
    let notes = [220.0f32, 261.63, 329.63, 440.0, 329.63, 261.63, 220.0, 164.81];
    let mut seed = 0x9E37_79B9u32;
    for (k, hz) in notes.iter().enumerate() {
        let start = (k as f32 * 0.45 * FS) as usize;
        let period = (FS / hz) as usize;
        let mut ring: Vec<f32> = (0..period)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                seed as f32 / u32::MAX as f32 * 2.0 - 1.0
            })
            .collect();
        for i in 0..(FS * 2.0) as usize {
            let j = i % period;
            let next = 0.5 * (ring[j] + ring[(j + 1) % period]) * 0.996;
            out[start + i] += ring[j] * 0.35;
            ring[j] = next;
        }
    }
    out
}

fn render(h: &mut PluginHarness<Heirloom>, input: &[f32], hold_at: usize) -> (Vec<f32>, Vec<f32>) {
    let (mut l, mut r) = (Vec::new(), Vec::new());
    for (b, chunk) in input.chunks(BLOCK).enumerate() {
        if b * BLOCK >= hold_at {
            h.set("hold", 1.0);
        }
        for ch in h.inputs_mut() {
            ch[..chunk.len()].copy_from_slice(chunk);
            ch[chunk.len()..].fill(0.0);
        }
        h.process(&[]);
        l.extend_from_slice(&h.outputs()[0][..chunk.len()]);
        r.extend_from_slice(&h.outputs()[1][..chunk.len()]);
    }
    (l, r)
}

#[test]
#[ignore = "writes audition WAVs to build/listen and prints levels; asserts nothing"]
fn audition_every_preset() {
    let input = phrase();
    let dry = rms(&input[..(FS * 4.0) as usize]);
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../build/listen");
    std::fs::create_dir_all(dir).unwrap();
    let params = HeirloomParams::default();
    let presets = Heirloom::presets(&params);
    println!("dry rms {:.1} dBFS", 20.0 * dry.log10());
    for preset in &presets {
        let name = preset.name;
        let run = |wet_only: bool| {
            let named = preset.values.iter().map(|(id, v)| (params.by_id(*id).expect("preset id").meta().name, *v));
            let mut h = named
                .chain(wet_only.then_some(("mix", 1.0)))
                .fold(PluginHarness::<Heirloom>::builder().sample_rate(FS).block_size(BLOCK), |b, (n, v)| b.set(n, v))
                .build();
            render(&mut h, &input, (FS * 8.0) as usize)
        };
        let (wl, _) = run(true);
        let (l, r) = run(false);
        let wet = rms(&wl[(FS * 1.0) as usize..(FS * 4.0) as usize]);
        let peak = l.iter().chain(&r).fold(0.0f32, |m, x| m.max(x.abs()));
        println!(
            "{name:<18} wet-only {:+5.1} dB re dry   out peak {:5.2}",
            20.0 * (wet / dry).log10(),
            peak
        );
        let file = format!("{dir}/heirloom-{}.wav", name.replace(" / ", "-").replace(' ', "_").to_lowercase());
        let audio = stave_host::wav::Audio { sample_rate: FS as u32, channels: vec![l, r] };
        stave_host::wav::write(std::path::Path::new(&file), &audio).unwrap();
    }
}
