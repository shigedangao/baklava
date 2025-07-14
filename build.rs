use std::path::PathBuf;

fn main() -> miette::Result<()> {
    let include_path = PathBuf::from(
        "/Users/marcintha/workspace/insightface/cpp-package/inspireface/build/inspireface-macos-apple-silicon-arm64/InspireFace/include",
    );
    // let other_include_cpp_path = PathBuf::from(
    //     "/Users/marcintha/workspace/insightface/cpp-package/inspireface/cpp/inspireface/include/inspireface"
    // );
    // let engine_include_path = PathBuf::from(
    //     "/Users/marcintha/workspace/insightface/cpp-package/inspireface/cpp/inspireface"
    // );
    // let third_party_include_path = PathBuf::from("/Users/marcintha/workspace/insightface/cpp-package/inspireface/3rdparty/yaml-cpp/include");
    // let inspire_cv_path = PathBuf::from("/Users/marcintha/workspace/insightface/cpp-package/inspireface/3rdparty/InspireCV/3rdparty/Eigen-3.4.0-Headers");

    let build = autocxx_build::Builder::new("src/main.rs", [&include_path]).build()?;
    build
        //.file("/Users/marcintha/workspace/insightface/cpp-package/inspireface/cpp/inspireface/c_api/inspireface.cc")
        // .file("/Users/marcintha/workspace/insightface/cpp-package/inspireface/cpp/inspireface/log.cpp")
        // .file("/Users/marcintha/workspace/insightface/cpp-package/inspireface/cpp/inspireface/engine/face_session.cpp")
        // .file("/Users/marcintha/workspace/insightface/cpp-package/inspireface/cpp/inspireface/middleware/timer.cpp")
        // .file("/Users/marcintha/workspace/insightface/cpp-package/inspireface/cpp/inspireface/feature_hub/feature_hub_db.cpp")
        // .file("/Users/marcintha/workspace/insightface/cpp-package/inspireface/cpp/inspireface/track_module/face_track_module.cpp")
        // .file("/Users/marcintha/workspace/insightface/cpp-package/inspireface/cpp/inspireface/track_module/tracker_optional/bytetrack/BYTETracker.cpp")
        // .file("/Users/marcintha/workspace/insightface/cpp-package/inspireface/cpp/inspireface/track_module/tracker_optional/bytetrack/tracker_utils.cpp")
        // .file("/Users/marcintha/workspace/insightface/cpp-package/inspireface/cpp/inspireface/track_module/tracker_optional/bytetrack/lapjv.cpp")
        .compile("inspire-face");

    println!("cargo::rustc-link-search=native=/Users/marcintha/workspace/insightface/cpp-package/inspireface/build/inspireface-macos-apple-silicon-arm64/InspireFace/lib");
    println!("cargo::rustc-link-lib=dylib=InspireFace");
    println!("cpargo:rerun-if-changed=src/main.rs");

    Ok(())
}
