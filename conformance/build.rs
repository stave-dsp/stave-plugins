//! The conformance tests link every plugin into one binary, and every
//! plugin's `stave::export!` defines the same entry points (`clap_entry`,
//! `GetPluginFactory`, …). Only a plugin's own cdylib calls them, so here
//! the linker keeps the first definition. macOS's linker never pulls the
//! duplicates in.
fn main() {
    match std::env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("linux") => println!("cargo:rustc-link-arg=-Wl,--allow-multiple-definition"),
        Ok("windows") => println!("cargo:rustc-link-arg=/FORCE:MULTIPLE"),
        _ => {}
    }
}
