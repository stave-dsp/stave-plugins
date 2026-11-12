//! Every check Vault owes (validation, no allocation, perf, the face's
//! laws, host behaviour), from one line. The hard per-machine CPU
//! ceilings are Vault's own laws: `budget.rs`.

stave_test::plugin_checks!(stave_vault_vst3::Vault, {
    perf: vec![PerfCase { name: "DEFAULT", set: &[] }],
});
