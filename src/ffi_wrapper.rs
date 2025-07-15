use autocxx::prelude::*;

include_cpp! {
    #include "inspireface.h"
    safety!(unsafe)
    generate!("HFFaceComparison")
    generate!("HFFaceFeatureWithRefExtractTo")
    generate!("HFCreateInspireFaceSessionOptional")
    generate!("HF_ENABLE_FACE_RECOGNITION")
    generate!("HFSession")
    generate!("HFDetectMode")
    generate!("HFCreateFaceFeature")
    generate!("HSUCCEED")
    generate!("HFLaunchInspireFace")
    generate!("HFCreateImageBitmapFromFilePath")
    generate!("HFCreateImageStreamFromImageBitmap")
    generate!("HFReleaseImageBitmap")
    generate!("HFImageBitmap")
    generate!("HFImageStream")
    generate!("HFRotation")
    generate!("HFExecuteFaceTrack")
    generate!("HFMultipleFaceData")
    generate!("HFGetTokens")
    generate!("HFFaceBasicToken")
    generate_pod!("HFFaceBasicTokenSlice")
    generate!("HFReleaseImageStream")
    generate!("HFloat")
}

pub use ffi::*;
