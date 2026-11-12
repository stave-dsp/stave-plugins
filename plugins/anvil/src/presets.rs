//! Factory presets — known places on the head. RHYTHM is the
//! load-time default (the metalcore brief); the others bracket the
//! space: the honest single tube path, and the drop-tuned extreme.

use stave::prelude::*;

use crate::params::AnvilParameters;

pub fn build_presets(parameters: &AnvilParameters) -> Vec<Preset> {
    vec![
        // The load-time brief, as a destination you can come back to.
        Preset::from_pairs(
            "RHYTHM",
            &[
                (parameters.output_level.id(), 0.0),
                (parameters.preamp_gain.id(), 18.0),
                (parameters.input_low_cut.id(), 120.0),
                (parameters.bass_tone.id(), 6.25),
                (parameters.midrange_tone.id(), 3.75),
                (parameters.treble_tone.id(), 6.666_667),
                (parameters.presence_tone.id(), 2.5),
                (parameters.depth_tone.id(), 2.5),
                (parameters.topology_separation_amount.id(), 0.5),
                (parameters.difference_tone.id(), 0.35),
                (parameters.high_frequency_emphasis.id(), 0.5),
                (parameters.power_pair_overdrive.id(), 12.0),
            ],
        ),
        // One honest tube path — the chug pumps the teeth, as in
        // every real amp (topology_separation_amount 0 = the classic rig alone).
        Preset::from_pairs(
            "CLASSIC",
            &[
                (parameters.output_level.id(), 0.0),
                (parameters.preamp_gain.id(), 14.0),
                (parameters.input_low_cut.id(), 90.0),
                (parameters.bass_tone.id(), 6.666_667),
                (parameters.midrange_tone.id(), 5.416_667),
                (parameters.treble_tone.id(), 6.25),
                (parameters.presence_tone.id(), 1.5),
                (parameters.depth_tone.id(), 1.5),
                (parameters.topology_separation_amount.id(), 0.0),
                (parameters.difference_tone.id(), 0.0),
                (parameters.high_frequency_emphasis.id(), 0.0),
                (parameters.power_pair_overdrive.id(), 8.0),
            ],
        ),
        // The drop-tuned extreme: full split, everything forward.
        Preset::from_pairs(
            "SLUDGE",
            &[
                (parameters.output_level.id(), 0.0),
                (parameters.preamp_gain.id(), 24.0),
                (parameters.input_low_cut.id(), 65.0),
                (parameters.bass_tone.id(), 7.5),
                (parameters.midrange_tone.id(), 5.0),
                (parameters.treble_tone.id(), 5.833_333),
                (parameters.presence_tone.id(), 2.0),
                (parameters.depth_tone.id(), 4.0),
                (parameters.topology_separation_amount.id(), 1.0),
                (parameters.difference_tone.id(), 0.6),
                (parameters.high_frequency_emphasis.id(), 1.0),
                (parameters.power_pair_overdrive.id(), 16.0),
            ],
        ),
    ]
}
