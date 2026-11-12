//! Eddy's typed parameter table. Ids are FOREVER — pick once, never
//! reuse or renumber. Groups: Delay (the line), Motion (the chorus),
//! Algorithm (the tabbed feedback laws + their per-tab knobs).

use stave::prelude::*;

param_enum! {
    /// The LFO character menu. Selector label truth — the editor pills
    /// and the host's step list read from HERE. The variants are
    /// RECIPES over the kernel basis (shape ∘ lag ∘ drift), never new
    /// oscillator kinds — see engine.rs.
    pub const MOTIONS: Motion = ["SINE", "TRI", "JUNO", "VIBE"];
}

param_enum! {
    /// The four delay algorithms — the editor's tabs. Each is a distinct
    /// feedback law over the same M/S delay substrate (engine/echo.rs);
    /// the per-tab knobs below only exist inside their algorithm.
    /// (Slot 2 was TRACK, an octave-divider follower — replaced by SHIFT
    /// 2026-07-05: it couldn't track real program and its per-lap wobble
    /// smeared under feedback.)
    pub const ALGOS: Algo = ["CIRCUIT", "MOTOR", "SHIFT", "ORBIT"];
}

param_enum! {
    /// CIRCUIT's regeneration cell — which physics sits in the loop.
    pub const CIRCUITS: CircuitKind = ["TUBE", "SCREAMER", "TAPE"];
}

param_enum! {
    /// The diffusion stage's four voices — the second tabbed section.
    /// The diffuser sits INSIDE the loop, on the read side: what you
    /// hear is what regenerates, so density compounds per lap and
    /// cranking DIFFUSE grows the delay into one integrated reverberant
    /// system (never early/late reflections bolted around a line).
    pub const DIFFUSIONS: DiffusionKind = ["PLATE", "PRISM", "DUST", "MESH"];
}

#[derive(Params)]
pub struct EddyParams {
    /// Output trim. Smoothing is DECLARED here (table-owned ramp),
    /// never hand-rolled in the engine.
    pub output: FloatParam,
    /// The framework gate owns click-free bypass; this carries the flag.
    pub bypass: BoolParam,
    /// Delay time. The slow declared glide IS the tape bend: moving
    /// the knob sweeps the read head and pitch-bends the tail.
    pub time: FloatParam,
    /// How much of the wet tail is fed back into the line — the echo
    /// train's decay. Capped below unity: this delay never runs away.
    pub feedback: FloatParam,
    /// Dry LEVEL — the untouched program's own knob. At 1 with wet
    /// at 0 the output is BIT-identical dry (a law test).
    /// (id 5 was `mix`, a blend — retired 2026-07-05 for these two
    /// independent levels; the id is never reused.)
    pub dry: FloatParam,
    /// Wet LEVEL — the echo train's own knob, independent of dry.
    pub wet: FloatParam,
    /// Chorus rate — how fast the read head wobbles.
    pub rate: FloatParam,
    /// Chorus depth — how far it wobbles (0 = the delay stands still).
    pub depth: FloatParam,
    /// LFO character: pristine shapes or the analog recipes.
    pub motion: EnumParam<Motion>,
    /// Where the wobble lives: 0 = on the MID (both ears together —
    /// solid center, mono-safe), 1 = on the SIDE (maximal width,
    /// center anchored). The phantom center survives either way.
    pub spread: FloatParam,
    /// How much analog character the recipe applies: 0 = pristine
    /// (JUNO becomes the raw triangle, VIBE the raw sine), 1 = the
    /// lamp glowing at full thermal lag. Pristine shapes (SINE/TRI)
    /// have no character to exaggerate and ignore it by definition.
    pub flavor: FloatParam,
    /// Which feedback law runs — the editor's tab. Host-visible, so
    /// tab choice automates and saves like any knob.
    pub algo: EnumParam<Algo>,
    /// CIRCUIT: the physics cell in the regeneration path.
    pub circuit: EnumParam<CircuitKind>,
    /// CIRCUIT: how hard each repeat drives the cell. 0 BYPASSES the
    /// cell — the loop is the classic law, bit for bit (a law test).
    pub heat: FloatParam,
    /// MOTOR: how much program energy loads the motor — the wet tail
    /// drags its own time (and pitch) flat, then recovers.
    pub sag: FloatParam,
    /// MOTOR: the platter's mass — how slowly the sag leans in and
    /// lets go.
    pub inertia: FloatParam,
    /// SHIFT: semitones each repeat is transposed by — +12 is the
    /// shimmer, negative spirals down. (id 16 was TRACK's amount —
    /// retired with the algorithm; the id is never reused.)
    pub shift: FloatParam,
    /// SHIFT: the transposer's grain window — the splice cadence.
    /// Small = tight flutter, large = washy; turning it bends the
    /// heads through its declared glide (the TIME-knob honesty).
    pub grain: FloatParam,
    /// SHIFT: how hard the machine hides each seam — at 1 every
    /// incoming head is phase-aligned by search (pristine), at 0 it
    /// lands wherever the sweep says (raw tape-splice warble).
    pub splice: FloatParam,
    /// ORBIT: degrees the echo image rotates per repeat (negative =
    /// the other way around). 0 = echoes hold their place.
    pub orbit: FloatParam,
    /// Which diffusion voice runs — the second tab bar.
    pub diffusion: EnumParam<DiffusionKind>,
    /// The diffusion stage's depth. 0 BYPASSES it — the loop is the
    /// undiffused law, bit for bit (a law test); 1 with feedback up
    /// is the full integrated reverb.
    pub diffuse: FloatParam,
    /// PLATE: the lattice's allpass strength — echo softening at low
    /// settings, full smear at the top.
    pub density: FloatParam,
    /// PLATE: how far the lattice delays WANDER (the Lexicon move —
    /// static lattices ring metallic; wandering ones bloom).
    pub wander: FloatParam,
    /// PRISM: where the chirp bends — low is the deep whale-tail
    /// (a first-order dispersive line ALWAYS lags lows; physics
    /// permits no other direction), high is the tight zap.
    pub bend: FloatParam,
    /// PRISM: how deep into the dispersive line each repeat travels
    /// (continuous — the tap slides along the cascade).
    pub smear: FloatParam,
    /// DUST: the cloud's radius — how far the micro-taps stray.
    pub spray: FloatParam,
    /// DUST: how many motes carry the echo — few coarse grains up to
    /// a fine dust.
    pub motes: FloatParam,
    /// MESH: the woven lines' lengths — small rooms to long strands.
    pub size: FloatParam,
    /// MESH: how hard the image-space rotations cross-couple the
    /// lines (and how much they re-circulate inside the mesh).
    pub weave: FloatParam,
    /// The field's OWN feedback — how long it rings after a repeat
    /// feeds it. This is what unchains the diffusion from the delay
    /// time: a 2-second echo can sit inside a continuous wash.
    pub hold: FloatParam,
    /// Field → line: how much of the field re-enters the
    /// regeneration. 0 = the field blooms AROUND an articulate
    /// delay; up = field content recirculates through the algorithm
    /// itself (shimmer feeding reverb feeding shimmer).
    pub bleed: FloatParam,
}

impl Default for EddyParams {
    fn default() -> Self {
        Self {
            output: FloatParam::new(ParamId(1), "output", Range::linear(-24.0, 24.0))
                .default(0.0)
                .unit(Unit::Db)
                .group("Output")
                .smooth(Smooth::ms(5.0)),
            bypass: BoolParam::bypass(ParamId(2)).group("Output"),
            time: FloatParam::new(ParamId(3), "time", Range::log(1.0, 2000.0))
                .default(350.0)
                .unit(Unit::Ms)
                .group("Delay")
                .smooth(Smooth::ms(80.0)),
            feedback: FloatParam::new(ParamId(4), "feedback", Range::linear(0.0, 0.95))
                .default(0.35)
                .group("Delay")
                .smooth(Smooth::ms(20.0)),
            dry: FloatParam::new(ParamId(18), "dry", Range::linear(0.0, 1.0))
                .default(1.0)
                .group("Delay")
                .smooth(Smooth::ms(5.0)),
            wet: FloatParam::new(ParamId(19), "wet", Range::linear(0.0, 1.0))
                .default(0.35)
                .group("Delay")
                .smooth(Smooth::ms(5.0)),
            rate: FloatParam::new(ParamId(6), "rate", Range::log(0.05, 10.0))
                .default(0.8)
                .group("Motion")
                .smooth(Smooth::ms(50.0)),
            depth: FloatParam::new(ParamId(7), "depth", Range::linear(0.0, 1.0))
                .default(0.0)
                .group("Motion")
                .smooth(Smooth::ms(50.0)),
            motion: EnumParam::new(ParamId(8), "motion", Motion(0)).group("Motion"),
            spread: FloatParam::new(ParamId(9), "spread", Range::linear(0.0, 1.0))
                .default(0.5)
                .group("Motion")
                .smooth(Smooth::ms(50.0)),
            flavor: FloatParam::new(ParamId(10), "flavor", Range::linear(0.0, 1.0))
                .default(0.5)
                .group("Motion")
                .smooth(Smooth::ms(50.0)),
            algo: EnumParam::new(ParamId(11), "algo", Algo(0)).group("Algorithm"),
            circuit: EnumParam::new(ParamId(12), "circuit", CircuitKind(0)).group("Algorithm"),
            heat: FloatParam::new(ParamId(13), "heat", Range::linear(0.0, 1.0))
                .default(0.0)
                .group("Algorithm")
                .smooth(Smooth::ms(50.0)),
            sag: FloatParam::new(ParamId(14), "sag", Range::linear(0.0, 1.0))
                .default(0.4)
                .group("Algorithm")
                .smooth(Smooth::ms(50.0)),
            inertia: FloatParam::new(ParamId(15), "inertia", Range::linear(0.0, 1.0))
                .default(0.5)
                .group("Algorithm")
                .smooth(Smooth::ms(50.0)),
            shift: FloatParam::new(ParamId(20), "shift", Range::linear(-12.0, 12.0))
                .default(12.0)
                .group("Algorithm")
                .smooth(Smooth::ms(50.0)),
            grain: FloatParam::new(ParamId(21), "grain", Range::log(16.0, 160.0))
                .default(64.0)
                .unit(Unit::Ms)
                .group("Algorithm")
                .smooth(Smooth::ms(50.0)),
            splice: FloatParam::new(ParamId(22), "splice", Range::linear(0.0, 1.0))
                .default(1.0)
                .group("Algorithm")
                .smooth(Smooth::ms(50.0)),
            orbit: FloatParam::new(ParamId(17), "orbit", Range::linear(-180.0, 180.0))
                .default(60.0)
                .group("Algorithm")
                .smooth(Smooth::ms(50.0)),
            diffusion: EnumParam::new(ParamId(23), "diffusion", DiffusionKind(0)).group("Diffusion"),
            diffuse: FloatParam::new(ParamId(24), "diffuse", Range::linear(0.0, 1.0))
                .default(0.0)
                .group("Diffusion")
                .smooth(Smooth::ms(50.0)),
            density: FloatParam::new(ParamId(25), "density", Range::linear(0.0, 1.0))
                .default(0.5)
                .group("Diffusion")
                .smooth(Smooth::ms(50.0)),
            wander: FloatParam::new(ParamId(26), "wander", Range::linear(0.0, 1.0))
                .default(0.5)
                .group("Diffusion")
                .smooth(Smooth::ms(50.0)),
            bend: FloatParam::new(ParamId(27), "bend", Range::linear(0.0, 1.0))
                .default(0.35)
                .group("Diffusion")
                .smooth(Smooth::ms(50.0)),
            smear: FloatParam::new(ParamId(28), "smear", Range::linear(0.0, 1.0))
                .default(0.5)
                .group("Diffusion")
                .smooth(Smooth::ms(50.0)),
            spray: FloatParam::new(ParamId(29), "spray", Range::linear(0.0, 1.0))
                .default(0.5)
                .group("Diffusion")
                .smooth(Smooth::ms(50.0)),
            motes: FloatParam::new(ParamId(30), "motes", Range::linear(0.0, 1.0))
                .default(0.6)
                .group("Diffusion")
                .smooth(Smooth::ms(50.0)),
            size: FloatParam::new(ParamId(31), "size", Range::linear(0.0, 1.0))
                .default(0.5)
                .group("Diffusion")
                .smooth(Smooth::ms(50.0)),
            weave: FloatParam::new(ParamId(32), "weave", Range::linear(0.0, 1.0))
                .default(0.5)
                .group("Diffusion")
                .smooth(Smooth::ms(50.0)),
            hold: FloatParam::new(ParamId(33), "hold", Range::linear(0.0, 1.0))
                .default(0.5)
                .group("Diffusion")
                .smooth(Smooth::ms(50.0)),
            bleed: FloatParam::new(ParamId(34), "bleed", Range::linear(0.0, 1.0))
                .default(0.0)
                .group("Diffusion")
                .smooth(Smooth::ms(50.0)),
        }
    }
}
