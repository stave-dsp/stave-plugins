//! Stave Halo: a high-headroom clean amplifier whose attack can remain cleaner
//! than its sustain.
//!
//! Halo is a head, not a cabinet or spatial effect. Its audio path is singular;
//! spectral analysis exists only in the control plane, where independent power
//! reservoirs prevent bass energy from automatically consuming the chime
//! reservoir.

mod controls;
mod editor;
mod engine;
mod params;
mod presets;

pub use engine::{Halo, HaloTaps};
pub use params::HaloParameters;

stave::export!(Halo, vst3_class_ids: {
    component: [0xB5D1B293, 0x88B54BE7, 0xA96D3431, 0xD624F842],
    controller: [0x49084B2F, 0xAD5D41A8, 0xBBC0A431, 0xB9378EE4],
});
