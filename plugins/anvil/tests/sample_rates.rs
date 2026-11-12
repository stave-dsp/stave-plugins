//! Every rate a host may pick, 1 kHz to 768 kHz (fractional included):
//! the output stays finite, never sits in subnormals, and sounds. (The
//! generic `any_sample_rate` check in checks.rs asks only for finite.)

use stave_anvil_vst3::Anvil;
use stave_test::PluginHarness;

const BLOCK_SIZE: usize = 256;
const RATES: [f32; 6] = [1_000.0, 44_100.0, 192_000.0, 352_800.0, 384_000.0, 768_000.0];

fn render_at(sample_rate: f32) -> Vec<f32> {
    let mut harness = PluginHarness::<Anvil>::new(sample_rate, BLOCK_SIZE);
    let mut rendered = Vec::new();
    let mut phase = 0.0f64;
    let mut block = vec![0.0f32; BLOCK_SIZE];
    for _ in 0..8 {
        for sample in block.iter_mut() {
            *sample = (0.5 * (phase * std::f64::consts::TAU).sin()) as f32;
            phase = (phase + 220.0 / sample_rate as f64).fract();
        }
        for channel in harness.inputs_mut() {
            channel.copy_from_slice(&block);
        }
        harness.process(&[]);
        for channel in harness.outputs() {
            rendered.extend_from_slice(channel);
        }
    }
    rendered
}

#[test]
#[cfg_attr(miri, ignore)]
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
