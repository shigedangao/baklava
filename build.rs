use std::env;

fn main() {
    let lib_path = env::var("INSIGHTFACE_PATH").expect("Expect library path to be defiend");
    // Our own shim.h (autocxx-friendly wrappers around functions that take a
    // `void**`) lives alongside the Rust sources in src/.
    let shim_include = env::current_dir().unwrap().join("src");
    let shim_include = shim_include.to_str().expect("Expect a valid src path");

    // Build the headers for rust
    autocxx_build::Builder::new(
        "src/ffi_wrapper.rs",
        [&format!("{lib_path}/include"), &shim_include.to_string()],
    )
        .build()
        .unwrap()
        .flag_if_supported("-std=c++17")
        .compile("inspire-face");

    let dylib_file_path = format!("{lib_path}/lib");
    println!("cargo::rustc-link-search=native={dylib_file_path}");
    println!("cargo::rustc-link-lib=dylib=InspireFace");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{dylib_file_path}");
    println!("cargo::rerun-if-changed=src/lib.rs");
    println!("cargo::rerun-if-changed=src/ffi_wrapper.rs");
    println!("cargo::rerun-if-changed=src/shim.h");
}
