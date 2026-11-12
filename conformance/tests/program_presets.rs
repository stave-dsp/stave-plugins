//! A plugin's factory presets reach a host through the VST3 program
//! parameter alone: names in order, and selecting one sets its values.

use stave::Plugin;
use stave_forge_vst3::{Forge, ForgeParams};
use stave_host::config::HostConfig;
use stave_host::instance::{apply_preset, presets, Instance};
use stave_host::plugin::LoadedPlugin;

#[test]
fn presets_are_listed_and_selected_through_the_program_parameter() {
    let declared = Forge::presets(&ForgeParams::default());
    let mut p = LoadedPlugin::from_entry_points(
        stave_host::entry_points!(stave_forge_vst3),
        HostConfig::default(),
    )
    .unwrap();
    let names: Vec<&str> = declared.iter().map(|d| d.name).collect();
    assert_eq!(presets(&p), names);

    let chosen = declared.last().unwrap();
    apply_preset(&mut p, chosen.name).unwrap();
    let params = p.params();
    for (id, plain) in &chosen.values {
        let c = params.iter().find(|c| c.id == *id).unwrap();
        assert!(
            (c.value - plain).abs() < 1e-6,
            "{}: {} vs {plain}",
            c.name,
            c.value
        );
    }
}
