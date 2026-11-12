//! Every plugin's identity, pinned: the CLAP id and the VST3 component
//! and controller class ids hosts key saved sessions on. These are the
//! values the v0.2.2 builds shipped (read back from the built bundles);
//! they never change. A plugin whose identity moved here would load as
//! a missing plugin in every saved session.


/// A VST3 class id as hosts print it: its sixteen bytes in hex.
fn hex(tuid: &[u8]) -> String {
    tuid.iter().map(|b| format!("{b:02X}")).collect()
}

macro_rules! identities {
    ($($name:ident = $plugin:ty, $clap:literal, $component:literal, $controller:literal;)*) => {$(
        #[test]
        fn $name() {
            let id = stave::identity::<$plugin>();
            assert_eq!(id.clap, $clap, "CLAP id");
            assert_eq!(hex(&id.vst3_component), $component, "VST3 component class id");
            assert_eq!(hex(&id.vst3_controller), $controller, "VST3 controller class id");
        }
    )*};
}

identities! {
    aeolian = stave_aeolian_vst3::Aeolian, "com.ryanbahan.stave.aeolian.vst3", "1109FB578C770B3FDE422D024153C104", "88E6EFD3F14BF1D72EADEF5823F72554";
    anvil = stave_anvil_vst3::Anvil, "com.ryanbahan.stave.anvil.vst3", "5A908C8884EC0CD91727EBCC38EB1EFE", "E2F5272C4B153E7CB777CA1096D452A6";
    chime = stave_chime_vst3::Chime, "com.ryanbahan.stave.chime.vst3", "3E92B7C14A5D4E88B1F0A6D222810001", "3E92B7C14A5D4E88B1F0A6D222810002";
    dynamo = stave_dynamo_vst3::Dynamo, "com.ryanbahan.stave.dynamo.vst3", "8A29C64A4FC494A70755565D7AFF1F4D", "7263AFDAD9CADF8925C02CF87B51E498";
    eddy = stave_eddy_vst3::Eddy, "com.ryanbahan.stave.eddy.vst3", "5A7FEDD96E5E4F889D6D7E1155E90001", "5A7FEDD96E5E4F889D6D7E1155E90002";
    engram = stave_engram_vst3::Engram, "com.ryanbahan.stave.engram.vst3", "9D173A5CD2BFAA66F030659FAA5CF54C", "98623D49E19F9168A90E8C84F7C87EDC";
    forge = stave_forge_vst3::Forge, "com.ryanbahan.stave.forge.vst3", "5A7FE1016E5E4F889D6D7E1155E82001", "5A7FE1016E5E4F889D6D7E1155E82002";
    halo = stave_halo_vst3::Halo, "com.ryanbahan.stave.halo.vst3", "B5D1B29388B54BE7A96D3431D624F842", "49084B2FAD5D41A8BBC0A431B9378EE4";
    heirloom = stave_heirloom_vst3::Heirloom, "com.ryanbahan.stave.heirloom.vst3", "68E5A59A63E215C202E460FC637F31D7", "997883ABFACDA95233F55080D9A58D26";
    multiband_comp = stave_multiband_comp_vst3::MultibandComp, "com.ryanbahan.stave.multibandcomp.vst3", "5A7FCC1C6E5E4F889D6D7E1155E76BE1", "5A7FCC1C6E5E4F889D6D7E1155E76BE2";
    opal = stave_opal_vst3::Opal, "com.ryanbahan.stave.opal.vst3", "6B47D2A991E34C778AF5B0E133920001", "6B47D2A991E34C778AF5B0E133920002";
    revenant = stave_revenant_vst3::Revenant, "com.ryanbahan.stave.revenant.vst3", "6774941E457E446D62BC0705061C8A4A", "CE9351551A4A48A16524FDBEA4DFB0B5";
    ribbon = stave_ribbon_vst3::Ribbon, "com.ryanbahan.stave.ribbon.vst3", "57D986BC6E8207EB51A73C99D1CD583C", "78955EAB3D6B1CFCE191FB7FCF075A71";
    saga = stave_saga_vst3::Saga, "com.ryanbahan.stave.saga.vst3", "C0765B4B46762B108645B824889E2C27", "039D76AD9D871B51582038D9EA200920";
    smear = stave_smear_vst3::Smear, "com.ryanbahan.stave.smear.vst3", "5A7FDD1D6E5E4F889D6D7E1155E76BF1", "5A7FDD1D6E5E4F889D6D7E1155E76BF2";
    tolex = stave_tolex_vst3::Tolex, "com.ryanbahan.stave.tolex.vst3", "EB796B35EDF8C2C4F9C39DCABD83B4C7", "5E690385BCCBE4AB9D21E046541F0AC3";
    vault = stave_vault_vst3::Vault, "com.ryanbahan.stave.vault.vst3", "B08D3A3D7965C2F26A8988F628E796EC", "937A36190C719553091740BAD89EAFD4";
}
