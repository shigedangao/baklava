use autocxx::prelude::*;

include_cpp! {
    #include "inspireface.h"
    #include "shim.h"
    safety!(unsafe)
    generate!("HFFaceComparison")
    generate!("HFFaceFeatureWithRefExtractTo")
    // These InspireFace functions take a `void**` out-param (PHFImageBitmap /
    // PHFImageStream / PHFSession) which autocxx cannot bind, so we go through
    // the inline wrappers declared in shim.h that return the handle instead.
    generate!("baklava_create_session_optional")
    generate!("baklava_create_image_bitmap_from_path")
    generate!("baklava_create_image_stream_from_bitmap")
    generate!("HF_ENABLE_FACE_RECOGNITION")
    generate_pod!("HFDetectMode")
    generate!("HFCreateFaceFeature")
    generate!("HSUCCEED")
    generate!("HFLaunchInspireFace")
    generate!("HFReleaseImageBitmap")
    generate_pod!("HFRotation")
    generate!("HFExecuteFaceTrack")
    generate!("HFMultipleFaceData")
    generate!("HFGetTokens")
    generate!("HFFaceBasicToken")
    generate_pod!("HFFaceBasicTokenSlice")
    generate!("HFReleaseImageStream")
    generate!("HFloat")
    generate!("HFReleaseInspireFaceSession")
    generate!("HFReleaseFaceFeature")
}

pub use ffi::*;
