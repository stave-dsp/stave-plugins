//! Revenant's typed parameter table — the face. Ids are FOREVER —
//! pick once, never reuse or renumber.
//!
//! ```text
//!   ( the organism )            [ SURPRISE ]
//!   LEARN  MEMORY  CHAOS  HORIZON  DREAM  MIX
//!   (bypass)        revenant              (sleep)
//! ```

use stave::prelude::*;

#[derive(Params)]
pub struct RevenantParams {
    /// Output trim (host-side; not on the face).
    pub output: FloatParam,
    /// Left footswitch. The framework gate owns click-free bypass.
    pub bypass: BoolParam,
    /// How fast it learns you (the readout's step size). 0 = it stops
    /// learning and keeps what it knows.
    pub learn: FloatParam,
    /// How long the network remembers: its neurons' leak, from quick
    /// (timbre) to slow (smeared, long-memory).
    pub memory: FloatParam,
    /// The network's gain round its own loops (spectral radius
    /// 0.6 … 1.35). Past the middle it starts to ring on its own.
    pub chaos: FloatParam,
    /// How far ahead it must guess what you will play, ms. Short, it
    /// shadows you; long, it has to imagine.
    pub horizon: FloatParam,
    /// How much it listens to itself instead of you.
    pub dream: FloatParam,
    /// Clean dry ↔ the revenant (bit-exact dry at 0).
    pub mix: FloatParam,
    /// Right footswitch: it stops listening and stops learning — and
    /// plays on from what it has learned, feeding on its own output.
    pub sleep: BoolParam,
}

impl Default for RevenantParams {
    fn default() -> Self {
        let unit = || Range::linear(0.0, 1.0);
        Self {
            output: FloatParam::new(ParamId(1), "output", Range::linear(-24.0, 24.0))
                .default(0.0)
                .unit(Unit::Db)
                .group("Output")
                .smooth(Smooth::ms(5.0)),
            bypass: BoolParam::bypass(ParamId(2)).group("Output"),
            learn: FloatParam::new(ParamId(3), "learn", unit())
                .default(0.6)
                .unit(Unit::Percent)
                .group("Mind")
                .smooth(Smooth::ms(50.0)),
            memory: FloatParam::new(ParamId(4), "memory", unit())
                .default(0.6)
                .unit(Unit::Percent)
                .group("Mind")
                .smooth(Smooth::ms(50.0)),
            chaos: FloatParam::new(ParamId(5), "chaos", unit())
                .default(0.45)
                .unit(Unit::Percent)
                .group("Mind")
                .smooth(Smooth::ms(50.0)),
            horizon: FloatParam::new(ParamId(6), "horizon", Range::log(0.25, 25.0))
                .default(3.0)
                .unit(Unit::Ms)
                .group("Mind"),
            dream: FloatParam::new(ParamId(7), "dream", unit())
                .default(0.0)
                .unit(Unit::Percent)
                .group("Mind")
                .smooth(Smooth::ms(50.0)),
            mix: FloatParam::new(ParamId(8), "mix", unit())
                .default(0.5)
                .unit(Unit::Percent)
                .group("Output")
                .smooth(Smooth::ms(10.0)),
            sleep: BoolParam::new(ParamId(9), "sleep", false)
                .group("Mind")
                .names("AWAKE", "ASLEEP"),
        }
    }
}
