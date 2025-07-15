use autocxx::c_void;
use autocxx::prelude::*;
use ffi_wrapper::{
    HF_ENABLE_FACE_RECOGNITION, HFCreateFaceFeature, HFCreateImageBitmapFromFilePath,
    HFCreateImageStreamFromImageBitmap, HFCreateInspireFaceSessionOptional, HFDetectMode,
    HFExecuteFaceTrack, HFFaceComparison, HFFaceFeature, HFFaceFeatureWithRefExtractTo,
    HFGetTokens, HFImageBitmap, HFImageStream, HFLaunchInspireFace, HFMultipleFaceData,
    HFReleaseImageBitmap, HFReleaseImageStream, HFSession, HSUCCEED,
};
use std::{
    ffi::{CString, c_void as StdCVoid},
    mem::{self},
};

#[allow(clippy::all)]
mod ffi_wrapper;

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

const SUCCESS: i64 = HSUCCEED as i64;

fn compare_image(img: Vec<CString>, model: CString) -> Result<f32, Box<dyn std::error::Error>> {
    let session: *mut StdCVoid = HFSession::default();
    let mut comparison_cosine_value: f32 = 0.;
    
    unsafe {
        if HFLaunchInspireFace(model.as_ptr()).0 != SUCCESS {
            return Err("Unable to load the model".into());
        }

        // Create a session ptr to convert it to an autocxx c_void pointer
        let mut session_ptr: *mut c_void = session as *mut c_void;
        let session_ptr_ptr: *mut *mut c_void = &mut session_ptr;

        // Initialize an HFMultipleFaceData structure in the way c++ would do
        let mut multiple_face_data: HFMultipleFaceData = mem::zeroed();

        // Create a session with the HF_ENABLE_FACE_RECOGNITION flag
        let res = HFCreateInspireFaceSessionOptional(
            c_int(HF_ENABLE_FACE_RECOGNITION as i32),
            HFDetectMode::HF_DETECT_MODE_ALWAYS_DETECT,
            c_int(1),
            c_int(-1),
            c_int(-1),
            session_ptr_ptr,
        );

        if res.0 != SUCCESS {
            return Err("Unable to load session".into());
        }

        // Initialize a vector of features to process two images the way c++ would do
        let mut features: Vec<HFFaceFeature> = vec![mem::zeroed(), mem::zeroed()];

        for (idx, feature) in &mut features.iter_mut().enumerate() {
            if HFCreateFaceFeature(feature).0 != SUCCESS {
                return Err("unable to create feature".into());
            }

            let img_bitmap: *mut StdCVoid = HFImageBitmap::default();
            let mut img_bitmap_ptr: *mut c_void = img_bitmap as *mut c_void;
            let img_bitmap_ptr_ptr: *mut *mut c_void = &mut img_bitmap_ptr;

            let stream: *mut StdCVoid = HFImageStream::default();
            let mut stream_ptr: *mut c_void = stream as *mut c_void;
            let stream_ptr_ptr: *mut *mut c_void = &mut stream_ptr;

            let img_tgt = img
                .get(idx)
                .ok_or("Image path not found")?;

            let res =
                HFCreateImageBitmapFromFilePath(img_tgt.as_ptr(), c_int(3), img_bitmap_ptr_ptr).0;

            if res != SUCCESS {
                return Err("Unable to process image".into());
            }

            if img_bitmap_ptr.is_null() || stream_ptr_ptr.is_null() {
                return Err("Image bitmap or stream pointer is null".into());
            }

            let res = HFCreateImageStreamFromImageBitmap(
                img_bitmap_ptr,
                ffi::HFRotation::HF_CAMERA_ROTATION_0,
                stream_ptr_ptr,
            ).0;
            if res != SUCCESS {
                return Err("Unable to create stream img".into());
            }

            if HFExecuteFaceTrack(session_ptr, stream_ptr, &mut multiple_face_data).0 != SUCCESS {
                return Err("Unable to run face track".into());
            }

            let tokens_slice = HFGetTokens(&mut multiple_face_data);
            let tokens_ptr = tokens_slice.ptr as *mut ffi::HFFaceBasicToken;
            let tokens_length = tokens_slice.len;

            // Construct the slices from the raw pointers
            let tokens = std::slice::from_raw_parts_mut(tokens_ptr, tokens_length as usize);
            let tk = tokens.get_mut(0).ok_or("Unable to get token")?;

            if session_ptr.is_null() || stream_ptr.is_null() {
                return Err("Session or stream pointer is null".into());
            }

            let ret = HFFaceFeatureWithRefExtractTo(session_ptr, stream_ptr, tk, feature);
            if ret != c_long(SUCCESS) {
                return Err("Unable to extract feature".into());
            }

            HFReleaseImageBitmap(img_bitmap_ptr);
            HFReleaseImageStream(stream_ptr);
        }

        let comparison_result = HFFaceComparison(&features[0], &features[1], &mut comparison_cosine_value);
        if comparison_result.0 != SUCCESS {
            return Err("Unable to compare image".into());
        }
    }

    Ok(comparison_cosine_value)
}

fn main() {
    let first_image = CString::new("./img1.png").unwrap();
    let second_image = CString::new("./img2.png").unwrap();
    let model = CString::new("./Megatron").unwrap();

    let res = compare_image(vec![first_image, second_image], model).unwrap();
    dbg!(res);
}
