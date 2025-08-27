use std::env;
use std::path::PathBuf;

fn main() {
    let lib_path = match env::var("INSIGHTFACE_PATH") {
        Ok(path) => PathBuf::from(path),
        Err(_) => {
            let current_dir = env::current_dir().unwrap();
            match env::var("DOCS_RS").is_ok() {
                // Use the curent_directory which include the "include" directory.
                true => current_dir,
                false => {
                    let local_path = format!(
                        "insightface/cpp-package/inspireface/build/inspireface-{}/InspireFace",
                        env::consts::OS
                    );

                    current_dir.join(local_path)
                }
            }
        }
    };

    let insightface_lib_str = lib_path.join("lib");
    let insightface_include = lib_path.join("include");

    // Build the headers for rust
    autocxx_build::Builder::new("src/ffi_wrapper.rs", [&insightface_include])
        .build()
        .unwrap()
        .flag_if_supported("-std=c++17")
        .compile("inspire-face");

    let dylib_file_path = insightface_lib_str
        .to_str()
        .expect("Expect to get the dylib path");

    println!("cargo::rustc-link-search=native={dylib_file_path}");
    println!("cargo::rustc-link-lib=dylib=InspireFace");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{dylib_file_path}");
    println!("cargo::rerun-if-changed=src/lib.rs");
}
