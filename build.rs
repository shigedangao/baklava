use std::path::PathBuf;
use std::{env, process::Command};

fn build_documentation() -> PathBuf {
    let out_dir = env::var("OUT_DIR").expect("Expect out_dir to be defined");
    Command::new("unzip")
        .arg("insightface_headers.zip")
        .arg("-d")
        .arg(&out_dir)
        .spawn()
        .expect("Expect unzip command to work")
        .wait()
        .expect("Expect cmd to unzip the headers");

    env::current_dir()
        .expect("Expect get current directory")
        .join(out_dir)
}

fn main() {
    let lib_path = match env::var("DOC_RS") {
        Ok(_) => build_documentation().to_str().unwrap().to_string(),
        Err(_) => env::var("INSIGHTFACE_PATH").expect("Expect library path to be defined"),
    };
    // Our own shim.h (autocxx-friendly wrappers around functions that take a
    // `void**`) lives alongside the Rust sources in src/.
    let shim_include = env::current_dir().unwrap().join("src");
    let shim_include = shim_include.to_str().expect("Expect a valid src path");

    // Build the headers for rust
    autocxx_build::Builder::new(
        "src/ffi_wrapper.rs",
        [format!("{lib_path}/include").as_str(), shim_include],
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
