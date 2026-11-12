//! The multiband engine: an LR4 split into three [`Band`]s, each a
//! compressor with its own detector, saturation and parallel blend,
//! summed back. An optional external sidechain is split at the same
//! crossovers to drive detection.

use stave::prelude::*;
use stave::telemetry::Scope;
use stave::Aux;

use crate::params::{MbParams, BANDS};

mod band;
pub(crate) mod dynamics;
mod splitter;

use band::{Band, BandControls};
use splitter::Splitter;

/// What the face reads: the rate the spectrum's axis is drawn at, and
/// each band's gain reduction in dB (≤ 0).
#[derive(Taps)]
pub struct MultibandTaps {
    pub sample_rate: Tap,
    pub gr_lo: Tap,
    pub gr_mid: Tap,
    pub gr_hi: Tap,
}

impl MultibandTaps {
    /// Each band's gain-reduction tap, low to high.
    pub const GR: [TapId; BANDS] = [Self::GR_LO, Self::GR_MID, Self::GR_HI];

    fn bands(&self) -> [&Tap; BANDS] {
        [&self.gr_lo, &self.gr_mid, &self.gr_hi]
    }
}

pub struct MultibandComp {
    params: Arc<MbParams>,
    sample_rate: f32,
    split: Splitter,
    /// The external sidechain's own split: same cutoffs, own state.
    sidechain_split: Splitter,
    sidechain_was_live: bool,
    crossovers: [Smoother; 2],
    crossover_hz: [f32; 2],
    controls: [BandControls; BANDS],
    bands: [Band; BANDS],
    telemetry: Arc<Telemetry>,
    taps: MultibandTaps,
    /// The face's spectrum feed: mono input and output on the capture
    /// wire (the plugin owns the capture; sized in `prepare`).
    scope: Scope,
}

/// Retune both splits wherever a cutoff moved.
fn set_crossovers(mut splits: [&mut Splitter; 2], current: &mut [f32; 2], hz: [f32; 2]) {
    if hz[0] != current[0] {
        for split in splits.iter_mut() {
            split.set_low(hz[0]);
        }
    }
    if hz[1] != current[1] {
        for split in splits.iter_mut() {
            split.set_high(hz[1]);
        }
    }
    *current = hz;
}

impl Plugin for MultibandComp {
    const INFO: Info = Info {
        id: "com.ryanbahan.stave.multibandcomp.vst3",
        name: "Stave Multiband",
        vendor: "Ryan Bahan",
        url: "https://github.com/stave-dsp/stave",
        description: "3-band multiband compressor with Linkwitz-Riley crossover.",
        features: &[Feature::AudioEffect, Feature::Stereo],
        ..Info::EFFECT
    };
    /// Preference order. Stereo + an external-sidechain aux first; plain
    /// stereo for hosts that won't wire an aux. A host picks a whole
    /// layout (VST3 setBusArrangements, CLAP audio-ports-config), never
    /// probes a predicate.
    const LAYOUTS: &'static [Layout] = &[
        Layout {
            aux: &[Aux {
                name: "sidechain",
                channels: Channels::Stereo,
            }],
            ..Layout::STEREO
        },
        Layout::STEREO,
    ];
    type Params = MbParams;

    fn new(params: &Arc<MbParams>, telemetry: &Telemetry) -> Self {
        telemetry.claim_capture();
        Self {
            sample_rate: 48_000.0,
            split: Splitter::new(params.xover_low.value(), params.xover_high.value()),
            sidechain_split: Splitter::new(params.xover_low.value(), params.xover_high.value()),
            sidechain_was_live: false,
            crossovers: [Smoother::new(&params.xover_low), Smoother::new(&params.xover_high)],
            crossover_hz: [f32::NAN; 2],
            controls: std::array::from_fn(|i| BandControls::new(&params.bands[i])),
            bands: std::array::from_fn(|_| Band::new()),
            telemetry: telemetry.shared(),
            taps: MultibandTaps::new(telemetry),
            scope: Scope::new(),
            params: params.clone(),
        }
    }

    fn prepare(&mut self, env: &Environment) {
        let p = &*self.params;
        self.sample_rate = env.sample_rate;
        self.scope.prepare(env.max_block);
        self.split.prepare(env.sample_rate);
        self.sidechain_split.prepare(env.sample_rate);
        self.sidechain_was_live = false;
        for smoother in &mut self.crossovers {
            smoother.prepare(env);
        }
        self.crossover_hz = [f32::NAN; 2];
        set_crossovers(
            [&mut self.split, &mut self.sidechain_split],
            &mut self.crossover_hz,
            [p.xover_low.value(), p.xover_high.value()],
        );
        for ((controls, band), bp) in self.controls.iter_mut().zip(&mut self.bands).zip(&p.bands) {
            controls.prepare(env);
            band.prepare(env.sample_rate, bp);
        }
    }

    /// The saturators' 4× FIR stack: constant across bands and settings,
    /// and the plug-in's only latency.
    fn latency(&self) -> u32 {
        self.bands[0].latency()
    }

    fn process(&mut self, block: &mut Block<'_>) {
        let p = &*self.params;
        for (band, bp) in self.bands.iter_mut().zip(&p.bands) {
            band.configure(bp.style.value(), bp.saturation.value());
        }
        let any_solo = p.bands.iter().any(|b| b.solo.value());
        let audible: [bool; BANDS] = std::array::from_fn(|i| {
            !p.bands[i].mute.value() && (p.bands[i].solo.value() || !any_solo)
        });

        let frames = block.frames();
        // The external sidechain, read in the lane loop straight off its
        // aux bus.
        let sidechain = block
            .aux(0)
            .filter(|aux| p.external_sidechain.value() && aux.channel_count() > 0)
            .map(|aux| (aux.channel(0), aux.channel(if aux.channel_count() > 1 { 1 } else { 0 })));
        let live = sidechain.is_some();
        if live && !self.sidechain_was_live {
            self.sidechain_split.reset();
        }
        self.sidechain_was_live = live;

        let Self {
            split,
            sidechain_split,
            crossovers,
            crossover_hz,
            controls,
            bands,
            scope,
            ..
        } = self;
        let [low, high] = crossovers;
        let mut low = low.ramp(block, &p.xover_low);
        let mut high = high.ramp(block, &p.xover_high);
        let [c0, c1, c2] = controls;
        let mut ramps = [
            c0.ramp(block, &p.bands[0]),
            c1.ramp(block, &p.bands[1]),
            c2.ramp(block, &p.bands[2]),
        ];
        let lanes = block.lanes::<2>();
        let stereo = match lanes.active() {
            0 => return,
            active => active > 1,
        };
        lanes.run(|fr| {
            set_crossovers(
                [&mut *split, &mut *sidechain_split],
                crossover_hz,
                [low.next(), high.next()],
            );
            let left = fr.input(0);
            let right = if stereo { fr.input(1) } else { left };
            let signal = split.split(left, right, stereo);
            let detect = if let Some((key_l, key_r)) = sidechain {
                let i = fr.index();
                sidechain_split.split(key_l[i], key_r[i], true)
            } else {
                signal
            };
            let mut out = (0.0, 0.0);
            for (i, band) in bands.iter_mut().enumerate() {
                let settings = ramps[i].next();
                let (l, r) = band.process(&settings, detect[i], signal[i], stereo);
                if audible[i] {
                    out.0 += l;
                    out.1 += r;
                }
            }
            fr.write(0, out.0);
            if stereo {
                fr.write(1, out.1);
            }
            let mono_out = if stereo { 0.5 * (out.0 + out.1) } else { out.0 };
            scope.write(fr.index(), 0.5 * (left + right), mono_out);
        });
        self.scope.publish(&self.telemetry, frames);
        self.taps.sample_rate.set(f64::from(self.sample_rate));
        for (tap, band) in self.taps.bands().into_iter().zip(&self.bands) {
            tap.set(band.reduction_db());
        }
    }

    fn face(params: &MbParams) -> Option<Face> {
        Some(crate::editor::face(params))
    }
}
