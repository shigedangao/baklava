use std::env;

fn main() -> miette::Result<()> {
    let include_path = env::var("INSPIRFACE_INCLUDE_PATH")
        .unwrap_or("/Users/marcinthaamnouay/workspace/insightface/cpp-package/inspireface/build/inspireface-macos-apple-silicon-arm64/InspireFace/include".into());

    let dylib_path = env::var("INSPIRFACE_DYLIB_PATH")
        .unwrap_or("/Users/marcinthaamnouay/workspace/insightface/cpp-package/inspireface/build/inspireface-macos-apple-silicon-arm64/InspireFace/lib".into());

    let build = autocxx_build::Builder::new("src/main.rs", [&include_path]).build()?;
    build
        .compile("inspire-face");

    println!("cargo::rustc-link-search=native={dylib_path}");
    println!("cargo::rustc-link-lib=dylib=InspireFace");
    println!("cpargo:rerun-if-changed=src/main.rs");

    Ok(())
}
