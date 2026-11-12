//! Forge's typed parameter table.
//!
//! Ids are FOREVER (docs/architecture.md): 1–19 in original declaration
//! order. Drive, bias, width and the trims glide per sample. Edge and
//! tone are `Smooth::Off`: they re-voice the stage tables and filters,
//! at block rate.
//! `bypass` is the host's bypass: the framework owns click-free bypass.

use stave::prelude::*;

param_enum! {
    /// The primary nonlinearity's harmonic recipe.
    pub const CHARACTERS: Character = ["SILK", "TRIODE", "CLASS AB", "TAPE", "DIODE", "FUZZ"];
}
param_enum! {
    /// The circuit topology.
    pub const STACKS: Stack = ["SINGLE", "CASCADE", "SCREAMER", "PUSH-PULL"];
}
param_enum! {
    /// The interstage filter placement.
    pub const VOICES: Voice = ["FLAT", "SCOOP", "BUMP", "BRIGHT", "DARK", "HONK"];
}
param_enum! {
    /// The program-dependent behaviour.
    pub const MEMORIES: Memory = ["NONE", "SAG", "BLOCKING", "SQUEEZE", "RIPPLE"];
}
param_enum! {
    /// The output EQ move.
    pub const POSTS: PostVoice = ["FLAT", "AIR", "LOW CUT", "WARM", "HI CUT", "DARK"];
}

/// Stack positions by NAME — the engine's mode gates read these
/// beside the selector that defines them.
pub(crate) mod stack {
    pub(crate) const SCREAMER: usize = 2;
    pub(crate) const PUSH_PULL: usize = 3;
}

#[derive(Params)]
pub struct ForgeParams {
    pub drive: FloatParam,
    pub character: EnumParam<Character>,
    pub edge: FloatParam,
    pub bias: FloatParam,
    pub stack: EnumParam<Stack>,
    pub filter: EnumParam<Voice>,
    pub post: EnumParam<PostVoice>,
    pub tone: FloatParam,
    pub memory: EnumParam<Memory>,
    pub depth: FloatParam,
    pub grit: FloatParam,
    pub iron: FloatParam,
    pub sub_mix: FloatParam,
    pub sub_mod: FloatParam,
    pub width: FloatParam,
    pub input: FloatParam,
    pub mix: FloatParam,
    pub output: FloatParam,
    pub bypass: BoolParam,
}

/// The stage frame's glide: drive and bias move the operating point of
/// a nonlinearity, where a step is a click.
const FRAME_GLIDE: Smooth = Smooth::ms(20.0);

impl Default for ForgeParams {
    fn default() -> Self {
        let unit = |id: u32, name: &'static str, default: f64, group: &'static str| {
            FloatParam::new(ParamId(id), name, Range::linear(0.0, 1.0))
                .default(default)
                .group(group)
                .smooth(Smooth::Off)
        };
        Self {
            drive: FloatParam::new(ParamId(1), "drive", Range::linear(0.0, 10.0))
                .default(4.0)
                .group("Stage 1")
                .smooth(FRAME_GLIDE),
            character: EnumParam::new(ParamId(2), "character", Character(1)).group("Stage 1"),
            edge: unit(3, "edge", 0.3, "Stage 1"),
            bias: unit(4, "bias", 0.0, "Stage 1").smooth(FRAME_GLIDE),
            stack: EnumParam::new(ParamId(5), "stack", Stack(1)).group("Topology"),
            filter: EnumParam::new(ParamId(6), "filter", Voice(0)).group("Topology"),
            post: EnumParam::new(ParamId(7), "post filter", PostVoice(0)).group("Topology"),
            tone: FloatParam::new(ParamId(8), "tone", Range::linear(-1.0, 1.0))
                .default(-0.2)
                .group("Topology")
                .smooth(Smooth::Off),
            memory: EnumParam::new(ParamId(9), "memory", Memory(1)).group("Texture"),
            depth: unit(10, "depth", 0.25, "Texture"),
            grit: unit(11, "grit", 0.0, "Texture"),
            iron: unit(12, "iron", 0.0, "Texture"),
            sub_mix: unit(13, "sub mix", 0.0, "Sub"),
            sub_mod: unit(14, "sub mod", 0.0, "Sub"),
            width: unit(15, "width", 0.5, "Stereo").smooth(FRAME_GLIDE),
            // The output trims are CONTINUOUS BINDINGS: smoothing is
            // declared here and rendered by the engine's
            // `Smoother`s, not hand-rolled.
            input: FloatParam::new(ParamId(16), "input", Range::linear(-24.0, 24.0))
                .default(0.0)
                .unit(Unit::Db)
                .group("Output")
                .smooth(Smooth::ms(5.0)),
            mix: FloatParam::new(ParamId(17), "mix", Range::linear(0.0, 1.0))
                .default(1.0)
                .group("Output")
                .smooth(Smooth::ms(5.0)),
            output: FloatParam::new(ParamId(18), "output", Range::linear(-24.0, 24.0))
                .default(0.0)
                .unit(Unit::Db)
                .group("Output")
                .smooth(Smooth::ms(5.0)),
            bypass: BoolParam::bypass(ParamId(19)).group("Output"),
        }
    }
}
