//! Every rate a host may pick, 1 kHz to 768 kHz (fractional included):
//! the output stays finite and never sits in subnormals.

use stave_core::{AudioBuffer, Plugin, ProcessCtx};

const BLOCK_SIZE: usize = 256;
const RATES: [f32; 6] = [1_000.0, 44_100.0, 192_000.0, 352_800.0, 384_000.0, 768_000.0];

fn render_at(sample_rate: f32) -> Vec<f32> {
    let mut plugin = stave_anvil_vst3::Anvil::new();
    plugin.activate(sample_rate, BLOCK_SIZE);
    let mut rendered = Vec::new();
    let (mut li, mut ri) = (vec![0.0f32; BLOCK_SIZE], vec![0.0f32; BLOCK_SIZE]);
    let (mut lo, mut ro) = (vec![0.0f32; BLOCK_SIZE], vec![0.0f32; BLOCK_SIZE]);
    let mut phase = 0.0f64;
    for _ in 0..8 {
        for (l, r) in li.iter_mut().zip(ri.iter_mut()) {
            *l = (0.5 * (phase * std::f64::consts::TAU).sin()) as f32;
            *r = *l;
            phase = (phase + 220.0 / sample_rate as f64).fract();
        }
        let mut ip = [core::ptr::null_mut(); 2];
        let mut op = [core::ptr::null_mut(); 2];
        let input = AudioBuffer::from_slices(&mut ip, [&mut li, &mut ri], BLOCK_SIZE);
        let output = AudioBuffer::from_slices(&mut op, [&mut lo, &mut ro], BLOCK_SIZE);
        let mut ctx = ProcessCtx {
            sample_rate,
            frames: BLOCK_SIZE,
            transport: None,
            input,
            output,
            aux: &[],
            params: stave_core::ParamBlock::empty(),
            events: &[],
        };
        plugin.process(&mut ctx);
        rendered.extend_from_slice(&lo);
        rendered.extend_from_slice(&ro);
    }
    rendered
}

#[test]
fn every_host_rate_renders_finite_normal_output() {
    for rate in RATES {
        let out = render_at(rate);
        for (i, s) in out.iter().enumerate() {
            assert!(s.is_finite(), "{rate} Hz: sample {i} is {s}");
            assert!(*s == 0.0 || s.is_normal(), "{rate} Hz: sample {i} is subnormal ({s:e})");
        }
        assert!(out.iter().any(|s| s.abs() > 1e-4), "{rate} Hz: silent");
    }
}
