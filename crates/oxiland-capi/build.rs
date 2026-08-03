//! Apply ELF symbol versioning and compile the variadic `librdf_log` shim.

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let version_script = std::path::Path::new(&manifest_dir).join("symbols.version");

    println!("cargo:rerun-if-changed=symbols.version");
    println!("cargo:rerun-if-changed=src/log_variadic.c");

    cc::Build::new()
        .file("src/log_variadic.c")
        .compile("oxiland_log_variadic");

    if target_os == "linux" {
        // GNU ld / compatible linkers: tag librdf_* with OXILAND_0.11 and
        // provide a LIBRDF_1.0.17 version node alias for Redland-shaped ELF loads.
        println!(
            "cargo:rustc-cdylib-link-arg=-Wl,--version-script={}",
            version_script.display()
        );
    }
}
