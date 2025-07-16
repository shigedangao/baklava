use miette::ErrReport;
use std::env;

fn main() -> miette::Result<()> {
    let include_path = env::var("INSPIRFACE_INCLUDE_PATH")
        .map_err(|_| ErrReport::msg("Unable to found the include path"))?;

    let dylib_path = env::var("INSPIRFACE_DYLIB_PATH")
        .map_err(|_| ErrReport::msg("Unable to found the library path"))?;

    autocxx_build::Builder::new("src/ffi_wrapper.rs", [&include_path])
        .build()?
        .compile("inspire-face");

    println!("cargo::rustc-link-search=native={dylib_path}");
    println!("cargo::rustc-link-lib=dylib=InspireFace");
    println!("cpargo:rerun-if-changed=src/main.rs");

    Ok(())
}
