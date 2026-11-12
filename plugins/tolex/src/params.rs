//! Tolex's typed parameter table. Ids are FOREVER — pick once,
//! never reuse or renumber. Ids 6–8, 13–14 and 16–18 are reserved.

use stave::prelude::*;

param_enum! {
    /// Driver diameter. Sets the cone area Sd and the moving-mass
    /// default the motor derives everything else from (engine/motor.rs
    /// SIZES table is the physics-side twin of this menu).
    pub const SIZES: SpeakerSize = ["8\"", "10\"", "12\"", "15\""];
}

param_enum! {
    /// Cabinet material — each is a panel-mode SET in the interior
    /// (engine/interior.rs PANELS is the physics twin): birch ply is
    /// stiff and ringy, pine loud and low, MDF dead.
    pub const MATERIALS: CabMaterial = ["BIRCH", "PINE", "MDF"];
}

param_enum! {
    /// How many cones share the baffle. Pure geometry at the mic
    /// (engine/drivers.rs): close-mic'd they collapse to one; pulled
    /// back they weave.
    pub const SPEAKERS: SpeakerCount = ["1x12", "2x12", "4x12"];
}

param_enum! {
    /// Mic models — each a signature DISTILLED from the reference
    /// corpus (median difference vs the SM57 over matched
    /// speaker/position groups; engine/microphone.rs). 57 = flat.
    pub const MICS: MicModel = ["57", "421", "58", "906", "SM7", "121", "160", "U47"];
}

param_enum! {
    /// The cabinet's back. CLOSED is the sealed air spring (the
    /// alignment shift is the motor's physics, not an EQ); OPEN
    /// frees the spring and opens the rear wrap path to the mic.
    pub const BACKS: CabBack = ["CLOSED", "OPEN"];
}

// DEFAULTS ARE BREAKDOWN (2026-07-26): load = the metalcore rig.
// The neutral bench lives on as the INIT preset and in every law's
// pinned bench sets.
#[derive(Params)]
pub struct TolexParams {
    /// Output trim. Smoothing is DECLARED here (table-owned ramp),
    /// never hand-rolled in the engine.
    pub output: FloatParam,
    /// The framework gate owns click-free bypass; this carries the flag.
    pub bypass: BoolParam,
    /// Driver diameter — the physical scale of everything.
    pub size: EnumParam<SpeakerSize>,
    /// Free-air resonance fs. Guitar drivers live 55–120 Hz.
    pub resonance: FloatParam,
    /// Total Q (Qts) at the resonance — how springy the suspension
    /// is against the motor's grip. House Qms is fixed; the electrical
    /// side derives from this (engine/motor.rs).
    pub q: FloatParam,
    /// How hard the amp swings the voice coil, in dB around an 8 V
    /// reference. Loudness-compensated by construction (the motor
    /// normalizes its passband by the same swing): this knob buys
    /// EXCURSION — compression and harmonics — not volume.
    pub drive: FloatParam,
    /// The amp's source impedance, 0 = solid-state grip (≈0 Ω),
    /// 1 = no-feedback tube output (≈40 Ω): the speaker's own
    /// impedance curve prints on the response. Real coupling no IR
    /// can render — a headline law.
    pub damping: FloatParam,
    /// The CONE — which fingerprint this speaker wears (the
    /// coordinate axis: anchors are fitted at specific cones; 0 is
    /// the house voice). Same physical law at every step.
    pub cone: FloatParam,
    /// Moving-mass scale (×Mms). Heavier cones break up lower and
    /// later; the whole modal fingerprint slides with it (a law).
    pub mass: FloatParam,
    /// How loud the cone's own voice is — the modal bank's level.
    /// 0 is a rigid piston (the fake); the default sits where the
    /// York population's ripple lives.
    pub breakup: FloatParam,
    /// Edge doping / damping: eats mode Q and prominence from the
    /// top of the band down, the way surround treatment does.
    pub doping: FloatParam,
    /// Closed or open back.
    pub back: EnumParam<CabBack>,
    /// Mic position across the cone, 0 = dust cap … 1 = cone edge.
    /// Macro: the corpus-calibrated −4.6 dB tilt walk. Micro: mode
    /// weights shift and the cap-cavity comb moves (~69% of peaks
    /// persist, a law).
    pub position: FloatParam,
    /// Interior volume in liters — the size of the air spring and
    /// the wrap path's scale.
    pub cab: FloatParam,
    /// What the box is built from — the panel modes' voice.
    pub material: EnumParam<CabMaterial>,
    /// Driver count — the weave.
    pub speakers: EnumParam<SpeakerCount>,
    /// Which microphone hears the cab. Signatures measured in
    /// context (corpus distillation), not from datasheets.
    pub model: EnumParam<MicModel>,
    /// Mic distance from the grille, in cm. Close = the direct field;
    /// pulled back = the open back's comb deepens (the rear path's
    /// level catches up with the front's).
    pub mic: FloatParam,
}

// The speaker, cab and mic construction rebuilds the model when it
// changes, so it does not glide; hosts may not automate it
// (`.not_automatable()` below).
impl Default for TolexParams {
    fn default() -> Self {
        Self {
            output: FloatParam::new(ParamId(1), "output", Range::linear(-24.0, 24.0))
                .default(0.0)
                .unit(Unit::Db)
                .group("Output")
                .smooth(Smooth::ms(5.0)),
            bypass: BoolParam::bypass(ParamId(2), "bypass").group("Output"),
            size: EnumParam::new(ParamId(3), "size", SpeakerSize(2)).group("Speaker"),
            resonance: FloatParam::new(ParamId(4), "resonance", Range::log(55.0, 120.0))
                .default(92.0)
                .unit(Unit::Hz)
                .group("Speaker")
                .not_automatable(),
            q: FloatParam::new(ParamId(5), "q", Range::linear(0.3, 1.2))
                .default(0.7)
                .group("Speaker")
                .not_automatable(),
            drive: FloatParam::new(ParamId(9), "drive", Range::linear(-12.0, 24.0))
                .default(6.0)
                .unit(Unit::Db)
                .group("Amp")
                .smooth(Smooth::ms(20.0)),
            damping: FloatParam::new(ParamId(10), "damping", Range::linear(0.0, 1.0))
                .default(0.3)
                .group("Amp")
                .not_automatable(),
            // Range extended 31 -> 32 the same day id 19 was minted
            // (pre-release; never in a shipped build): slot 32+ are
            // MEASURED anchors, 0-31 the invented space.
            cone: FloatParam::new(ParamId(19), "cone", Range::linear(0.0, 32.0))
                .default(21.0)
                .group("Speaker")
                .not_automatable(),
            mass: FloatParam::new(ParamId(6), "mass", Range::log(0.5, 2.0))
                .default(1.05)
                .group("Speaker")
                .not_automatable(),
            breakup: FloatParam::new(ParamId(7), "breakup", Range::linear(0.0, 1.0))
                .default(0.6)
                .group("Speaker")
                .not_automatable(),
            doping: FloatParam::new(ParamId(8), "doping", Range::linear(0.0, 1.0))
                .default(0.38)
                .group("Speaker")
                .not_automatable(),
            back: EnumParam::new(ParamId(11), "back", CabBack(0)).group("Cab"),
            position: FloatParam::new(ParamId(16), "position", Range::linear(0.0, 1.0))
                .default(0.15)
                .group("Mic")
                .not_automatable(),
            cab: FloatParam::new(ParamId(12), "cab", Range::log(25.0, 200.0))
                .default(170.0)
                .group("Cab")
                .not_automatable(),
            material: EnumParam::new(ParamId(13), "material", CabMaterial(0)).group("Cab"),
            speakers: EnumParam::new(ParamId(14), "speakers", SpeakerCount(2)).group("Cab"),
            model: EnumParam::new(ParamId(17), "model", MicModel(1)).group("Mic"),
            mic: FloatParam::new(ParamId(15), "mic", Range::log(0.5, 60.0))
                .default(2.5)
                .group("Mic")
                .smooth(Smooth::ms(30.0)),
        }
    }
}
