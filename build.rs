use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let lib_path = match env::var("INSIGHTFACE_PATH") {
        Ok(path) => PathBuf::from(path),
        Err(_) => {
            let current_dir = env::current_dir().unwrap();
            match env::var("DOCS_RS").is_ok() {
                true => {
                    let out_dir =
                        env::var("OUT_DIR").expect("Expect to found the out_dir variable");

                    // The headers can be passed from the "zipped" file "insightface_headers.zip"
                    Command::new("unzip")
                        .arg("insightface_headers.zip")
                        .arg("-d")
                        .arg(&out_dir)
                        .spawn()
                        .expect("Expect unzip command to work");

                    current_dir.join(out_dir)
                }
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
