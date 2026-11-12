//! Stave Vault. **Start here**: this file is the plugin's
//! root; everything below is one click away.
//!
//! ```text
//! lib.rs      ← you are here: the modules and the export
//! params.rs   every knob, with its forever-stable id
//! presets.rs  factory presets — the character space's corners
//! editor.rs   the face, a pure function of params
//! engine/     the plugin: mod.rs (`impl Plugin for Vault`, identity
//!               and orchestration), tank.rs (the
//!               8-line Hadamard FDN), diffuser.rs (input allpasses)
//! tests/      checks.rs (validation, no-alloc, perf, the face's laws),
//!               editor.rs, and the reverb's OWN laws (properties.rs — T60 is
//!               a measured promise, passivity, bit-dry mix zero)
//! ```
//!
//! STATUS: plugins/vault/docs/design.md — predelay → image-source ROOM (early) →
//! diffused second-order feed → 8-line FDN TANK (late) + MODAL
//! bank (LF standing waves) → lean/width/duck field → bit-dry mix.

mod editor;
mod engine;
mod params;
mod presets;

pub use engine::Vault;
pub use params::VaultParams;

stave::export!(Vault, vst3_class_ids: {
    component: [0xB08D3A3D, 0x7965C2F2, 0x6A8988F6, 0x28E796EC],
    controller: [0x937A3619, 0x0C719553, 0x091740BA, 0xD89EAFD4],
});
