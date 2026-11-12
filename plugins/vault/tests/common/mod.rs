//! The law suites' shared render loop: Vault in a `PluginHarness`,
//! fed block by block (inputs zero-padded), every output frame kept.

#![allow(dead_code)]

use stave_test::PluginHarness;
use stave_vault_vst3::Vault;

/// Vault prepared at `sample_rate` for blocks of `block`, starting
/// from `set` (plain values, by parameter name).
pub fn harness(set: &[(&str, f64)], sample_rate: f32, block: usize) -> PluginHarness<Vault> {
    set.iter()
        .fold(PluginHarness::<Vault>::builder().sample_rate(sample_rate).block_size(block), |b, (n, v)| b.set(n, *v))
        .build()
}

/// Run `in_l`/`in_r` through `h`, then `extra_blocks` blocks of
/// silence; `automate` sets one parameter (block index, name, plain
/// value) as automation does, before that block runs.
pub fn run(
    h: &mut PluginHarness<Vault>,
    in_l: &[f32],
    in_r: &[f32],
    extra_blocks: usize,
    automate: Option<(usize, &str, f64)>,
) -> (Vec<f32>, Vec<f32>) {
    let block = h.block_size();
    let blocks = in_l.len().max(in_r.len()).div_ceil(block) + extra_blocks;
    let (mut left, mut right) = (Vec::with_capacity(blocks * block), Vec::with_capacity(blocks * block));
    for b in 0..blocks {
        if let Some((at, name, v)) = automate {
            if b == at {
                h.set(name, v);
            }
        }
        for (src, dst) in [in_l, in_r].into_iter().zip(h.inputs_mut()) {
            for (i, x) in dst.iter_mut().enumerate() {
                *x = src.get(b * block + i).copied().unwrap_or(0.0);
            }
        }
        h.process(&[]);
        left.extend_from_slice(&h.outputs()[0]);
        right.extend_from_slice(&h.outputs()[1]);
    }
    (left, right)
}

/// Render the same signal into both ears at 48 kHz in 256-frame
/// blocks, plus `extra_blocks` of silence: (left, right).
pub fn render_stereo(set: &[(&str, f64)], input: &[f32], extra_blocks: usize) -> (Vec<f32>, Vec<f32>) {
    render_stereo_lr(set, input, input, extra_blocks)
}

/// [`render_stereo`] with independent L/R inputs.
pub fn render_stereo_lr(set: &[(&str, f64)], in_l: &[f32], in_r: &[f32], extra_blocks: usize) -> (Vec<f32>, Vec<f32>) {
    run(&mut harness(set, 48_000.0, 256), in_l, in_r, extra_blocks, None)
}

/// [`render_stereo`], left ear only.
pub fn render(set: &[(&str, f64)], input: &[f32], extra_blocks: usize) -> Vec<f32> {
    render_stereo(set, input, extra_blocks).0
}
