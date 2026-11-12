//! Every rate a host may pick, 1 kHz to 768 kHz (fractional included):
//! the output stays finite and never sits in subnormals.

use stave_halo_vst3::Halo;
use stave_test::PluginHarness;

const BLOCK_SIZE: usize = 256;
const RATES: [f32; 6] = [
    1_000.0, 44_100.0, 192_000.0, 352_800.0, 384_000.0, 768_000.0,
];

fn render_at(sample_rate: f32) -> Vec<f32> {
    let mut plugin = PluginHarness::<Halo>::new(sample_rate, BLOCK_SIZE);
    let mut rendered = Vec::new();
    let mut phase = 0.0f64;
    for _ in 0..8 {
        let mut block = [0.0f32; BLOCK_SIZE];
        for s in &mut block {
            *s = (0.5 * (phase * std::f64::consts::TAU).sin()) as f32;
            phase = (phase + 220.0 / sample_rate as f64).fract();
        }
        for channel in plugin.inputs_mut() {
            channel.copy_from_slice(&block);
        }
        plugin.process(&[]);
        for channel in plugin.outputs() {
            rendered.extend_from_slice(channel);
        }
    }
    rendered
}

#[test]
fn every_host_rate_renders_finite_normal_output() {
    for rate in RATES {
        let out = render_at(rate);
        for (i, s) in out.iter().enumerate() {
            assert!(s.is_finite(), "{rate} Hz: sample {i} is {s}");
            assert!(
                *s == 0.0 || s.is_normal(),
                "{rate} Hz: sample {i} is subnormal ({s:e})"
            );
        }
        assert!(out.iter().any(|s| s.abs() > 1e-4), "{rate} Hz: silent");
    }
}
