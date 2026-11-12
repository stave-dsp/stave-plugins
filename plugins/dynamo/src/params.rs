//! Dynamo's typed parameter table. Ids are FOREVER — pick once,
//! never reuse or renumber.

use stave::prelude::*;
use stave::route::RouteParams;

param_enum! {
    /// THE ROTOR's pole count — commutation harmonics without
    /// leaving the pitch.
    pub const POLES: PoleCount = ["1", "2", "3", "4", "6", "8", "12"];
}

param_enum! {
    /// How the exciter touches the wire: GRIND never lets go (the
    /// driven machine), STRIKE contacts for ~20 ms and lets the
    /// body sing, BOW is stick-slip friction (self-limiting,
    /// organic sustain).
    pub const CONTACTS: Contact = ["GRIND", "STRIKE", "BOW"];
}

param_enum! {
    /// The encoder wheel: a 16-slot pattern on the shaft gating the
    /// cycle — SOLID is the analog machine, PCM the digital one.
    pub const WHEELS: Wheel =
        ["SOLID", "HALF", "QUARTERS", "ALTERNATE", "RATCHET", "SPARSE", "MORSE", "PCM"];
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
    pub const TARGETS: ModTarget = [
        "PITCH", "LEVEL", "DECAY",
        // The MACROS (APPEND-only): global-scope targets — one
        // motion animates the machine, not sixteen private copies.
        "FORCE", "MASS", "MEMORY", "COUPLE", "STEAM", "CHAOS",
    ];
}

// Default voicing is the NAMESAKE: a struck bell — zero sustain
// (the note fades even while held; decay is the body), long release
// (letting go lets it ring out). Organ/pad behavior is one knob away
// (sustain up).
#[derive(Params)]
pub struct DynamoParams {
    /// Attack: ARRIVAL time to full level (the Adsr contract).
    pub attack: FloatParam,
    /// Decay toward sustain (time constant).
    pub decay: FloatParam,
    /// Where a held note settles.
    pub sustain: FloatParam,
    /// The ring after note-off (time constant).
    pub release: FloatParam,
    /// THE ROTOR: spin-up mass, load sag, poles, encoder wheel.
    pub mass: FloatParam,
    pub sag: FloatParam,
    pub poles: EnumParam<PoleCount>,
    pub wheel: EnumParam<Wheel>,
    /// THE WIRES: ring time (T60), stiffness (partial stretch),
    /// memory (the hysteretic head's magnetism). The machine is ONE
    /// unit — the body is its only voice (id 54 `blend`, the
    /// rotor-direct mix, is RETIRED: never reuse).
    pub ring: FloatParam,
    pub stiff: FloatParam,
    pub memory: FloatParam,
    /// Stereo width: the detuned pair splits left/right.
    pub width: FloatParam,
    /// Exciter contact mode.
    pub contact: EnumParam<Contact>,
    /// Machine noise INTO the wires: brush arcing + bearing rumble.
    pub grit: FloatParam,
    /// THE BRIDGE: every voice terminates on ONE nonlinear bridge —
    /// energy bleeds between sounding notes (sympathetic strings).
    pub couple: FloatParam,
    /// THE SUPPLY: boiler size. Small boilers droop under chords
    /// and recover in the rests; the machine goes breathy and flat.
    pub steam: FloatParam,
    /// THE ESCAPEMENT: the exciter's clock. 0 = disengaged (struck
    /// notes ring and die); low = ENTRAINED (ticks phase-locked to
    /// the wire's own period — self-sustaining, an impossible
    /// EBow); high = CHAOTIC (skipped and doubled teeth, the
    /// broken clock).
    pub chaos: FloatParam,
    /// FORCE: how hard the machine is driven (the sixth macro).
    pub force: FloatParam,
    /// THE LIFE SYSTEM — the machine's involuntary sounds,
    /// each a distinct physical subsystem driven by machine STATE
    /// and voiced through the same wires and bridge (never a
    /// parallel layer):
    /// * HUM — supply ripple; deepens as the boiler strains.
    pub hum: FloatParam,
    /// * BEARING — rotation noise; rumble and brush hiss that
    ///   follow the ACTUAL rotor speed (a stopped machine is
    ///   silent; spin-up is audible).
    pub bearing: FloatParam,
    /// * CLANK — contact collision modes; the dense inharmonic
    ///   burst of two bodies meeting, at every note contact.
    pub clank: FloatParam,
    /// * CRACKLE — media aging; pops whose RATE rides heat and
    ///   magnetization (a hot, magnetized machine crackles).
    pub crackle: FloatParam,
    /// * TENSION — real-string nonlinearity: loud notes bend sharp
    ///   with their own body energy and settle as they decay.
    pub tension: FloatParam,
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
    pub routes: [RouteParams<ModSource, ModTarget>; 10],
    /// Output trim. Smoothing is DECLARED here (table-owned ramp).
    pub output: FloatParam,
    /// The framework gate owns click-free bypass; this carries the flag.
    pub bypass: BoolParam,
}

impl Default for DynamoParams {
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
            mass: FloatParam::new(ParamId(5), "mass", Range::log(1.0, 2_000.0))
                .default(60.0)
                .unit(Unit::Ms)
                .group("Rotor")
                .smooth(Smooth::Off),
            sag: FloatParam::new(ParamId(20), "sag", Range::linear(0.0, 1.0))
                .default(0.25)
                .group("Rotor")
                .smooth(Smooth::Off),
            poles: EnumParam::new(ParamId(48), "poles", PoleCount(3)).group("Rotor"),
            wheel: EnumParam::new(ParamId(49), "wheel", Wheel(0)).group("Rotor"),
            ring: FloatParam::new(ParamId(50), "ring", Range::log(0.05, 10.0))
                .default(0.8)
                .group("Wires")
                .smooth(Smooth::Off),
            stiff: FloatParam::new(ParamId(52), "stiff", Range::linear(0.0, 1.0))
                .default(0.15)
                .group("Wires")
                .smooth(Smooth::Off),
            memory: FloatParam::new(ParamId(53), "memory", Range::linear(0.0, 1.0))
                .default(0.0)
                .group("Wires")
                .smooth(Smooth::Off),
            width: FloatParam::new(ParamId(55), "width", Range::linear(0.0, 1.0))
                .default(0.8)
                .group("Wires")
                .smooth(Smooth::Off),
            contact: EnumParam::new(ParamId(56), "contact", Contact(0)).group("Rotor"),
            grit: FloatParam::new(ParamId(57), "grit", Range::linear(0.0, 1.0))
                .default(0.15)
                .group("Rotor")
                .smooth(Smooth::Off),
            couple: FloatParam::new(ParamId(58), "couple", Range::linear(0.0, 1.0))
                .default(0.15)
                .group("Machine")
                .smooth(Smooth::Off),
            steam: FloatParam::new(ParamId(59), "steam", Range::linear(0.0, 1.0))
                .default(0.75)
                .group("Machine")
                .smooth(Smooth::Off),
            chaos: FloatParam::new(ParamId(60), "chaos", Range::linear(0.0, 1.0))
                .default(0.0)
                .group("Machine")
                .smooth(Smooth::Off),
            force: FloatParam::new(ParamId(61), "force", Range::linear(0.0, 1.0))
                .default(0.5)
                .group("Machine")
                .smooth(Smooth::Off),
            hum: FloatParam::new(ParamId(62), "hum", Range::linear(0.0, 1.0))
                .default(0.1)
                .group("Life")
                .smooth(Smooth::Off),
            bearing: FloatParam::new(ParamId(63), "bearing", Range::linear(0.0, 1.0))
                .default(0.12)
                .group("Life")
                .smooth(Smooth::Off),
            clank: FloatParam::new(ParamId(64), "clank", Range::linear(0.0, 1.0))
                .default(0.15)
                .group("Life")
                .smooth(Smooth::Off),
            crackle: FloatParam::new(ParamId(65), "crackle", Range::linear(0.0, 1.0))
                .default(0.05)
                .group("Life")
                .smooth(Smooth::Off),
            tension: FloatParam::new(ParamId(66), "tension", Range::linear(0.0, 1.0))
                .default(0.1)
                .group("Life")
                .smooth(Smooth::Off),
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
            routes: core::array::from_fn(|i| {
                RouteParams::new(
                    ParamId(if i < 4 {
                        10 * (i as u32 + 1)
                    } else {
                        100 + 10 * (i as u32 - 3)
                    }),
                    i + 1,
                    "Mod",
                )
            }),
            output: FloatParam::new(ParamId(6), "output", Range::linear(-24.0, 24.0))
                .default(0.0)
                .unit(Unit::Db)
                .group("Output")
                .smooth(Smooth::ms(5.0)),
            bypass: BoolParam::bypass(ParamId(7), "bypass").group("Output"),
        }
    }
}
