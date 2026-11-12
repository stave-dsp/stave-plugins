//! THE RATE LAW — the cabinet is the same cabinet at every sample
//! rate. All coefficients are rate-derived by construction (house
//! numeric convention); this law PROVES it: the rendered 1/6-oct
//! trend at 44.1 and 96 kHz must match the 48 kHz reference within
//! 1 dB rms / 2 dB max over 100 Hz-8 kHz, at a working sheet
//! (open back, position off-cap, pine, 4x12 — every subsystem in
//! the path).

use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use stave_test::PluginHarness;
use stave_tolex_vst3::Tolex;

const BLOCK: usize = 256;

fn render_ir(fs: f32, n: usize) -> Vec<f32> {
    let mut builder = PluginHarness::<Tolex>::builder().block_size(BLOCK);
    for (nm, v) in [
        ("back", 1.0),
        ("position", 0.6),
        ("material", 1.0),
        ("speakers", 2.0),
        ("model", 4.0),
        ("mic", 12.0),
        ("cone", 7.0),
    ] {
        builder = builder.set(nm, v);
    }
    let mut h = builder.sample_rate(fs).build();
    let mut out = Vec::with_capacity(n);
    let mut in_l = vec![0.0f32; BLOCK];
    for b in 0..(n / BLOCK) {
        in_l.fill(0.0);
        if b == 0 {
            in_l[0] = 0.01;
        }
        for ch in h.inputs_mut() {
            ch.copy_from_slice(&in_l);
        }
        h.process(&[]);
        out.extend_from_slice(&h.outputs()[0]);
    }
    out
}

fn trend(ir: &[f32], fs: f32, grid: &[f64]) -> Vec<f64> {
    const FFT_N: usize = 65_536;
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(FFT_N);
    let mut buf: Vec<Complex<f32>> = ir.iter().map(|x| Complex::new(*x, 0.0)).collect();
    buf.resize(FFT_N, Complex::new(0.0, 0.0));
    fft.process(&mut buf);
    let power: Vec<f64> = buf[..FFT_N / 2]
        .iter()
        .map(|c| f64::from(c.norm_sqr()))
        .collect();
    let bin_hz = f64::from(fs) / FFT_N as f64;
    let half = 2.0f64.powf(1.0 / 12.0);
    let mut out: Vec<f64> = grid
        .iter()
        .map(|f| {
            let b0 = ((f / half) / bin_hz).floor().max(1.0) as usize;
            let b1 = (((f * half) / bin_hz).ceil() as usize)
                .min(power.len() - 1)
                .max(b0 + 1);
            let mean = power[b0..=b1].iter().sum::<f64>() / (b1 - b0 + 1) as f64;
            10.0 * mean.max(1e-30).log10()
        })
        .collect();
    let sel: Vec<usize> = grid
        .iter()
        .enumerate()
        .filter(|(_, f)| (800.0..=1_250.0).contains(*f))
        .map(|(i, _)| i)
        .collect();
    let m = sel.iter().map(|i| out[*i]).sum::<f64>() / sel.len() as f64;
    for o in out.iter_mut() {
        *o -= m;
    }
    out
}

#[test]

#[cfg_attr(miri, ignore)]
fn the_cabinet_survives_the_clock() {
    let grid: Vec<f64> = (0..400)
        .map(|i| 90.0 * (8_000.0f64 / 90.0).powf(i as f64 / 399.0))
        .collect();
    let reference = trend(&render_ir(48_000.0, 24_064), 48_000.0, &grid);
    for fs in [44_100.0f32, 96_000.0] {
        let n = ((fs * 0.5) as usize / BLOCK) * BLOCK;
        let t = trend(&render_ir(fs, n), fs, &grid);
        let diffs: Vec<f64> = reference.iter().zip(&t).map(|(a, b)| a - b).collect();
        let rms = (diffs.iter().map(|d| d * d).sum::<f64>() / diffs.len() as f64).sqrt();
        let max = diffs.iter().fold(0.0f64, |a, d| a.max(d.abs()));
        assert!(
            rms <= 1.0 && max <= 2.0,
            "{fs} Hz drifts from 48 k: rms {rms:.2} dB, max {max:.2} dB"
        );
    }
}
