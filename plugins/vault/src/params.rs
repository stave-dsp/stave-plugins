//! Vault's typed parameter table. Ids are FOREVER — pick once,
//! never reuse or renumber. RETIRED ids (removed with the tape
//! purge, 2026-07-21 — never reuse): 9 (wow), 10 (drive);
//! 8 (motion — baked into the engines, 2026-07-21: an anti-
//! metallic health parameter, not a musical decision); 23/24/25
//! (launch/bloom/lift — folded into DISTANCE's low end, DUCK, and
//! a SIZE-scaled bake, 2026-07-21: the behaviors live where their
//! names already were).
//!
//! The PRISTINE↔LOFI axis is deliberately not one knob: DAMP is
//! the room's absorption, MOTION is the tank's chorus, WOW is slow
//! tape-style drift, DRIVE is loop saturation. All four at zero =
//! the pristine hall; dialed together they walk the machine down
//! to lofi without ever leaving the same physical tank.

use stave::prelude::*;

param_enum! {
    /// THE TOPOLOGY — which MACHINE this is, not a flavor of one
    /// machine (the arc's ratifying critique: one skeleton, many
    /// shaders = every preset is the same room). ROOM is the
    /// image-source + FDN + modes machine; PLATE is a dedicated
    /// 2D-membrane skeleton (short dispersive lines, instant
    /// density, damper decay — no geometry stages at all).
    pub const TOPOLOGIES: Topology = ["ROOM", "PLATE", "MEMORY", "DETONATE", "VOID"];
}

param_enum! {
    /// What the WALLS are made of — the room's construction, not a
    /// post-EQ: per-bounce absorption and brightness in the early
    /// field, bass ring and damping in the late (scene.rs owns the
    /// table). PLASTER is the pre-materials voicing exactly.
    pub const MATERIALS: WallMaterial =
        ["WOOD", "PLASTER", "STONE", "GLASS", "DRAPE", "METAL"];
}

#[derive(Params)]
pub struct VaultParams {
    /// Output trim. Smoothing is DECLARED here (table-owned ramp),
    /// never hand-rolled in the engine.
    pub output: FloatParam,
    /// The framework gate owns click-free bypass; this carries the flag.
    pub bypass: BoolParam,
    /// Dry/wet. mix 0 is BIT-dry (the parallel-chain contract).
    pub mix: FloatParam,
    /// Room scale: multiplies every tank line's length. The engine
    /// GLIDES length changes (slew-capped read heads — turning the
    /// room bends pitch like tape, never clicks).
    pub size: FloatParam,
    /// Decay time (T60) — a MEASURED promise, not a flavor name:
    /// the t60 law pins the dialed value against the rendered tail.
    pub decay: FloatParam,
    /// Wet onset delay. Glided by declaration (tape-bend, no click).
    pub predelay: FloatParam,
    /// The room's absorption: HF loss per trip through the tank.
    pub damp: FloatParam,
    /// Wet stereo width, 0..2: narrow → natural (1) → artificially
    /// wide (side boost + slight mid dip). Id 11 (the old 0..1
    /// range) is RETIRED — ranges are frozen, never bent.
    pub width: FloatParam,
    /// WHERE YOU STAND across the room's width. Off-center, the
    /// near wall answers earlier and stronger and each ear wears
    /// its own coloration — L/R balance EMERGES from the geometry,
    /// it is not panned on.
    pub stand: FloatParam,
    /// THE ROOM's voice in the wet field: the image-source early
    /// reflections' level. The late tank always grows out of the
    /// room's answer; this dial is how much of the answer you hear
    /// raw.
    pub early: FloatParam,
    /// Low-band decay MULTIPLIER (the bass ratio — real halls ring
    /// their bass 1.1-1.4x longer; tight rooms cut it): below
    /// ~350 Hz the tail decays at decay x bass.
    pub bass: FloatParam,
    /// TONE is the WALLS now, not a post-EQ: it scales the room's
    /// material cutoffs, darkening/brightening the early taps AND
    /// the late field through the tank's feed (the tank eats what
    /// the walls serve). DAMP remains the decay's darkening over
    /// time.
    pub tone: FloatParam,
    /// Scattering density: input diffusers AND the in-loop
    /// diffusion together. Low = discrete vintage echoes (a
    /// CHARACTER, not a defect); high = smooth wash.
    pub diffuse: FloatParam,
    /// DISPERSION — a stiff medium's frequency-dependent wave
    /// speed (metal plates and springs chirp; air does not): a
    /// first-order allpass cascade in every tank loop. 0 = a room.
    pub stiff: FloatParam,
    /// THE DUCK — the wet compresses under the DRY's envelope and
    /// blooms into the gaps (the companding trick that lets a
    /// reverb run LOUD and stay clear: reactiveness, not wash).
    pub duck: FloatParam,
    /// The walls' construction (see MATERIALS).
    pub material: EnumParam<WallMaterial>,
    /// Which machine (see TOPOLOGIES). Id 22.
    pub topology: EnumParam<Topology>,
    /// THE MEMORY's ladder (MEMORY machine only): energy leaks UP
    /// the resonator grid one octave at a time — the remembered
    /// chord climbs as it sustains. Shimmer with no pitch shifter,
    /// unconditionally stable (the ladder only goes up and out).
    /// Id 26.
    pub ascend: FloatParam,
    /// THE FUSE (DETONATE machine only): the burst's envelope
    /// shape — 0 = instant detonation that decays, 1 = a reverse
    /// swell that builds INTO the cut. Id 27.
    pub fuse: FloatParam,
    /// THE DAWN (VOID machine only): the send swells IN — sources
    /// rise into the field instead of splashing against it. Id 28.
    pub dawn: FloatParam,
    /// WHERE THE STAGE STANDS, front to back: 0 = right in front
    /// of you (intimate — long first-reflection gap, bright,
    /// early-heavy), 1 = the far wall (deep — short gap, dark,
    /// wash-heavy). One geometry moves every cue together.
    pub distance: FloatParam,
}

impl Default for VaultParams {
    fn default() -> Self {
        Self {
            output: FloatParam::new(ParamId(1), "output", Range::linear(-24.0, 24.0)).default(0.0)
                .unit(Unit::Db)
                .group("Output")
                .smooth(Smooth::ms(5.0)),
            bypass: BoolParam::bypass(ParamId(2)).group("Output"),
            mix: FloatParam::new(ParamId(3), "mix", Range::linear(0.0, 1.0)).default(0.32)
                .group("Field")
                .smooth(Smooth::ms(20.0)),
            size: FloatParam::new(ParamId(4), "size", Range::log(0.3, 3.0)).default(1.0)
                .group("Space")
                .smooth(Smooth::Off),
            decay: FloatParam::new(ParamId(5), "decay", Range::log(100.0, 30_000.0)).default(2_400.0)
                .unit(Unit::Ms)
                .group("Space")
                .smooth(Smooth::Off),
            predelay: FloatParam::new(ParamId(6), "predelay", Range::linear(0.0, 250.0)).default(12.0)
                .unit(Unit::Ms)
                .group("Space")
                .smooth(Smooth::ms(80.0)),
            damp: FloatParam::new(ParamId(7), "high energy", Range::linear(0.0, 1.0)).default(0.35)
                .group("Color")
                .smooth(Smooth::Off),
            width: FloatParam::new(ParamId(19), "width", Range::linear(0.0, 2.0)).default(1.0)
                .group("Field")
                .smooth(Smooth::ms(20.0)),
            stand: FloatParam::new(ParamId(18), "stand", Range::linear(-1.0, 1.0)).default(0.2)
                .group("Field")
                .smooth(Smooth::Off),
            early: FloatParam::new(ParamId(12), "early", Range::linear(0.0, 1.0)).default(0.55)
                .group("Space")
                .smooth(Smooth::ms(20.0)),
            bass: FloatParam::new(ParamId(13), "low energy", Range::log(0.25, 4.0)).default(1.0)
                .group("Color")
                .smooth(Smooth::Off),
            tone: FloatParam::new(ParamId(14), "tone", Range::linear(-1.0, 1.0)).default(0.0)
                .group("Color")
                .smooth(Smooth::Off),
            diffuse: FloatParam::new(ParamId(15), "diffuse", Range::linear(0.0, 1.0)).default(0.7)
                .group("Color")
                .smooth(Smooth::Off),
            stiff: FloatParam::new(ParamId(16), "whip", Range::linear(0.0, 1.0)).default(0.0)
                .group("Color")
                .smooth(Smooth::Off),
            duck: FloatParam::new(ParamId(17), "duck", Range::linear(0.0, 1.0)).default(0.0)
                .group("Field")
                .smooth(Smooth::Off),
            material: EnumParam::new(ParamId(20), "material", WallMaterial(1)).group("Color"),
            topology: EnumParam::new(ParamId(22), "topology", Topology(0)).group("Space"),
            ascend: FloatParam::new(ParamId(26), "ascend", Range::linear(0.0, 1.0)).default(0.0)
                .group("Space"),
            fuse: FloatParam::new(ParamId(27), "fuse", Range::linear(0.0, 1.0)).default(0.25)
                .group("Space"),
            dawn: FloatParam::new(ParamId(28), "dawn", Range::linear(0.0, 1.0)).default(0.4)
                .group("Space"),
            distance: FloatParam::new(ParamId(21), "distance", Range::linear(0.0, 1.0)).default(0.7)
                .group("Space")
                .smooth(Smooth::Off),
        }
    }
}
