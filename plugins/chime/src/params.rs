//! Chime's typed parameter table. Ids are FOREVER — pick once,
//! never reuse or renumber.

use stave::prelude::*;
use stave::route::RouteParams;

param_enum! {
    /// The oscillator's shape — pure sine, or the polyBLEP saw.
    pub const SHAPES: Shape = ["SINE", "SAW"];
}

param_enum! {
    /// Modulation sources. OFF disables the
    /// slot. Per-voice: VEL, KEY, ENV (the voice's own envelope),
    /// SPREAD (a stable per-note random — unison detune generalized
    /// to any target). Global: LFO, HEAT (a slow fatigue integrator
    /// of playing intensity — components warming up), SYMP (the
    /// OTHER voices' energy bleeding in, sympathetic-string style).
    pub const SOURCES: ModSource =
        ["OFF", "LFO", "VEL", "KEY", "ENV", "SPREAD", "HEAT", "SYMP"];
}

param_enum! {
    /// The LFO's waveform. SINE..SQR are the dsp shape basis; S&H
    /// and DRIFT are RandomWalk recipes (stepped renewal / eased walk).
    pub const LFO_SHAPES: LfoShapeP = ["SINE", "TRI", "SAW", "SQR", "S&H", "DRIFT"];
}

param_enum! {
    /// Rate mode: free Hz from the knob, or tempo-locked divisions.
    pub const LFO_SYNC: LfoSync =
        ["FREE", "1/1", "1/2", "1/4", "1/8", "1/16", "1/4T", "1/8T"];
}

param_enum! {
    /// Phase mode: FREE runs one global phase (chords shimmer);
    /// NOTE restarts each voice's phase at note-on (attacks agree).
    pub const LFO_TRIG: LfoTrig = ["FREE", "NOTE"];
}

param_enum! {
    /// Modulation destinations — the DECLARED allowlist (never "any
    /// param"): pitch in ±12 st, level in ±12 dB, decay stretched
    /// up to 4× either way.
    pub const TARGETS: ModTarget = ["PITCH", "LEVEL", "DECAY"];
}

// Default voicing is the NAMESAKE: a struck bell — zero sustain
// (the note fades even while held; decay is the body), long release
// (letting go lets it ring out). Organ/pad behavior is one knob away
// (sustain up).
#[derive(Params)]
pub struct ChimeParams {
    /// Attack: ARRIVAL time to full level (the Adsr contract).
    pub attack: FloatParam,
    /// Decay toward sustain (time constant).
    pub decay: FloatParam,
    /// Where a held note settles.
    pub sustain: FloatParam,
    /// The ring after note-off (time constant).
    pub release: FloatParam,
    /// The oscillator shape.
    pub shape: EnumParam<Shape>,
    /// The global LFO's rate (the matrix's clock).
    pub lfo_rate: FloatParam,
    pub lfo_shape: EnumParam<LfoShapeP>,
    pub lfo_sync: EnumParam<LfoSync>,
    /// Onset ramp: the LFO fades in over this many ms after
    /// note-on (a player's vibrato, not a machine's).
    pub lfo_rise: FloatParam,
    pub lfo_trig: EnumParam<LfoTrig>,
    /// The modulation routes: seven ordinary params per slot (see
    /// `RouteParams`), so automation, state and presets cover them.
    #[param(group)]
    pub routes: [RouteParams<ModSource, ModTarget>; 4],
    /// Output trim. Smoothing is DECLARED here (table-owned ramp).
    pub output: FloatParam,
    /// The framework gate owns click-free bypass; this carries the flag.
    pub bypass: BoolParam,
}

impl Default for ChimeParams {
    fn default() -> Self {
        Self {
            attack: FloatParam::new(ParamId(1), "attack", Range::log(0.5, 2_000.0))
                .default(5.0)
                .unit(Unit::Ms)
                .group("Envelope")
                .smooth(Smooth::Off),
            decay: FloatParam::new(ParamId(2), "decay", Range::log(1.0, 4_000.0))
                .default(800.0)
                .unit(Unit::Ms)
                .group("Envelope")
                .smooth(Smooth::Off),
            sustain: FloatParam::new(ParamId(3), "sustain", Range::linear(0.0, 1.0))
                .default(0.0)
                .group("Envelope")
                .smooth(Smooth::Off),
            release: FloatParam::new(ParamId(4), "release", Range::log(2.0, 8_000.0))
                .default(1_200.0)
                .unit(Unit::Ms)
                .group("Envelope")
                .smooth(Smooth::Off),
            shape: EnumParam::new(ParamId(5), "shape", Shape(0)).group("Voice"),
            lfo_rate: FloatParam::new(ParamId(8), "lfo rate", Range::log(0.05, 12.0))
                .default(1.2)
                .unit(Unit::Hz)
                .group("Mod")
                .smooth(Smooth::Off),
            lfo_shape: EnumParam::new(ParamId(9), "lfo shape", LfoShapeP(0)).group("Mod"),
            lfo_sync: EnumParam::new(ParamId(10), "lfo sync", LfoSync(0)).group("Mod"),
            lfo_rise: FloatParam::new(ParamId(18), "lfo rise", Range::linear(0.0, 2_000.0))
                .default(0.0)
                .unit(Unit::Ms)
                .group("Mod")
                .smooth(Smooth::Off),
            lfo_trig: EnumParam::new(ParamId(19), "lfo trig", LfoTrig(0)).group("Mod"),
            routes: core::array::from_fn(|i| RouteParams::new(ParamId(10 * (i as u32 + 1)), i + 1, "Mod")),
            output: FloatParam::new(ParamId(6), "output", Range::linear(-24.0, 24.0))
                .default(0.0)
                .unit(Unit::Db)
                .group("Output")
                .smooth(Smooth::ms(5.0)),
            bypass: BoolParam::bypass(ParamId(7)).group("Output"),
        }
    }
}
