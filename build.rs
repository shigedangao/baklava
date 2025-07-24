use miette::ErrReport;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

// Constant
const DEP_DIR: &str = "insightface/cpp-package/inspireface";

/// Copy the generated dynamic library dylib to rust deps path. This allows to not need to specify the DYLD_LIBRARY_PATH
/// during development (can also work in release). In the case if this needs to run in a different path then
/// the DYLD_LIBRARY_PATH must need to be specified.
///
/// # Arguments
///
/// * `dylib_path` - PathBuf
fn copy_dylib(dylib_path: PathBuf) {
    let mut target_path = PathBuf::from(env::var("OUT_DIR").expect("Expect to get the OUT_DIR"));

    // We pop 3 times in order to get to the /target/{mode} folder
    for _ in 0..3 {
        target_path.pop();
    }

    // Create the lib folder
    target_path.push("deps");

    if !target_path.exists() {
        fs::create_dir(&target_path).expect("Expect to create lib folder");
    }

    // Copy the libInspireFace.dylib to the lib path
    let mut dylib_lib_path = target_path.clone();
    dylib_lib_path.push("libInspireFace.dylib");

    fs::copy(dylib_path, dylib_lib_path).expect("Expect to copy the libInspireFace file");
}

fn main() -> miette::Result<()> {
    let current_dir = env::current_dir().expect("Expect to get the current directory");

    // Triggering the build of the insightface dependency from build.rs
    let mut lib_path = current_dir.clone();
    lib_path.push(DEP_DIR);

    if !lib_path.exists() {
        return Err(ErrReport::msg(
            "Unable to found the path of the inspireface library",
        ));
    }

    let mut include_path = lib_path.clone();
    include_path.push(format!(
        "build/inspireface-{}/InspireFace/include",
        env::consts::OS
    ));

    if !include_path.exists() {
        // Change the current directory
        env::set_current_dir(&lib_path).expect("Expect to be able to change the current dir");

        // Run the command to build the library
        let status = Command::new("sh")
            .arg("command/build.sh")
            .status()
            .expect("Unable to build the inspireface library {err}");

        if !status.success() {
            panic!("Expect to have build the library");
        }

        env::set_current_dir(current_dir)
            .expect("Set to current directory in order to generate the bindings");
    }

    let mut dylib_path = lib_path.clone();
    dylib_path.push(format!(
        "build/inspireface-{}/InspireFace/lib",
        env::consts::OS
    ));

    let dylib_path_str = dylib_path
        .as_os_str()
        .to_str()
        .expect("Expect to get dylib path");

    // Build the headers for rust
    autocxx_build::Builder::new("src/ffi_wrapper.rs", [&include_path])
        .build()?
        .compile("inspire-face");

    let dylib_file_path = format!("{dylib_path_str}/libInspireFace.dylib");

    // Copy the library to the target
    copy_dylib(PathBuf::from(dylib_file_path));

    println!("cargo::rustc-link-search=native={dylib_path_str}");
    println!("cargo::rustc-link-lib=dylib=InspireFace");
    println!("cargo::rerun-if-changed=src/lib.rs");

    Ok(())
}
