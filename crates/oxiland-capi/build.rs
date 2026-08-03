//! Apply ELF symbol versioning and compile the variadic `librdf_log` shim.

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR");
    let version_script = std::path::Path::new(&manifest_dir).join("symbols.version");
    let archive = std::path::Path::new(&out_dir).join("liboxiland_log_variadic.a");

    println!("cargo:rerun-if-changed=symbols.version");
    println!("cargo:rerun-if-changed=src/log_variadic.c");

    // Disable automatic `rustc-link-lib` so we can force-load the whole archive:
    // nothing in Rust references `librdf_log` by name, so the default archive
    // link would GC the object and the cdylib would not export the symbol.
    cc::Build::new()
        .file("src/log_variadic.c")
        .cargo_metadata(false)
        .compile("oxiland_log_variadic");

    println!("cargo:rustc-link-search=native={out_dir}");
    match target_os.as_str() {
        "macos" | "ios" => {
            // rustc's macOS cdylib export list + dead_strip would otherwise drop
            // an unreferenced C object even after -force_load.
            println!(
                "cargo:rustc-cdylib-link-arg=-Wl,-force_load,{}",
                archive.display()
            );
            println!("cargo:rustc-cdylib-link-arg=-Wl,-u,_librdf_log");
            println!("cargo:rustc-cdylib-link-arg=-Wl,-exported_symbol,_librdf_log");
        }
        "windows" => {
            println!(
                "cargo:rustc-cdylib-link-arg=/WHOLEARCHIVE:{}",
                archive.display()
            );
        }
        _ => {
            println!("cargo:rustc-link-lib=static:+whole-archive=oxiland_log_variadic");
        }
    }

    if target_os == "linux" {
        // GNU ld / compatible linkers: tag librdf_* with OXILAND_0.11 and
        // provide a LIBRDF_1.0.17 version node alias for Redland-shaped ELF loads.
        println!(
            "cargo:rustc-cdylib-link-arg=-Wl,--version-script={}",
            version_script.display()
        );
    }
}
