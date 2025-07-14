use autocxx::prelude::*;
use ffi::{
    HFFaceFeature,
    HFCreateInspireFaceSessionOptional,
    HF_ENABLE_FACE_RECOGNITION,
    HFDetectMode,
    HFSession,
    HFCreateFaceFeature,
    HFCreateImageStreamFromImageBitmap,
    HFExecuteFaceTrack,
    HFFaceFeatureWithRefExtractTo,
    HFReleaseImageBitmap,
    HFReleaseImageStream,
    HFFaceComparison,
    HFLaunchInspireFace
};
use autocxx::c_void;
use std::{ffi::{c_void as StdCVoid, CString}, mem::{self}};

use crate::ffi::{HFCreateImageBitmapFromFilePath, HFGetTokens, HFImageBitmap, HFImageStream, HFMultipleFaceData, HSUCCEED};

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
    generate!("HFReleaseFaceFeature")
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

fn compare_image(img: Vec<CString>) -> Result<(), Box<dyn std::error::Error>> {
    // std::ffi void
    let session: *mut StdCVoid = HFSession::default();
    let model = CString::new("./Megatron")?;
    
    unsafe {
        let ret = HFLaunchInspireFace(model.as_ptr());
        if ret != c_long(HSUCCEED as i64) {
            return Err("Unable to load the model".into());
        }

        let mut session_ptr: *mut c_void = session as *mut c_void;
        let session_ptr_ptr: *mut *mut c_void = &mut session_ptr; 

        let mut multiple_face_data: HFMultipleFaceData = mem::zeroed();

        let ret = HFCreateInspireFaceSessionOptional(
                c_int(HF_ENABLE_FACE_RECOGNITION as i32),
                HFDetectMode::HF_DETECT_MODE_ALWAYS_DETECT,
                c_int(1), 
                c_int(-1), 
                c_int(-1), 
                session_ptr_ptr
            );

        if ret != c_long(HSUCCEED as i64) {
            return Err("Unable to load session".into());
        } 

        let mut features: Vec<HFFaceFeature> = vec![mem::zeroed(), mem::zeroed()];
        for (idx, feature) in &mut features.iter_mut().enumerate() {
            let ret = HFCreateFaceFeature(feature);
            if ret != c_long(HSUCCEED as i64) {
                return Err("unable to create feature".into())
            }

            let img_bitmap: *mut StdCVoid = HFImageBitmap::default();
            let mut img_bitmap_auto: *mut c_void = img_bitmap as *mut c_void;
            let img_bitmap_auto_ptr: *mut *mut c_void = &mut img_bitmap_auto;

            let stream: *mut StdCVoid = HFImageStream::default();
            let mut stream_ptr: *mut c_void = stream as *mut c_void;
            let stream_ptr_ptr: *mut *mut c_void = &mut stream_ptr;

            let img_tgt = img.get(idx).unwrap();
            let ret = HFCreateImageBitmapFromFilePath(img_tgt.as_ptr(), c_int(3), img_bitmap_auto_ptr);
            if ret != c_long(HSUCCEED as i64) {
                return Err("Unable to process image".into());
            }

            let ret = HFCreateImageStreamFromImageBitmap(img_bitmap_auto, ffi::HFRotation::HF_CAMERA_ROTATION_0, stream_ptr_ptr);
            if ret != c_long(HSUCCEED as i64) {
                return Err("Unable to create stream img".into());
            }

            let ret = HFExecuteFaceTrack(session_ptr, stream_ptr, &mut multiple_face_data);
            if ret != c_long(HSUCCEED as i64) {
                return Err("Unable to run face track".into());
            }

            let tokens_slice = HFGetTokens(&mut multiple_face_data);
            let tokens_ptr = tokens_slice.ptr as *mut ffi::HFFaceBasicToken;
            let tokens_length = tokens_slice.len;

            let tokens = std::slice::from_raw_parts_mut(tokens_ptr, tokens_length as usize);
            let tk = tokens.get_mut(0).unwrap();
            
            let ret = HFFaceFeatureWithRefExtractTo(
                session_ptr,
                stream_ptr,
                tk,
                feature,
            );
            if ret != c_long(HSUCCEED as i64) {
                return Err("Unable to extract feature".into());
            }

            HFReleaseImageBitmap(img_bitmap_auto);
            HFReleaseImageStream(stream_ptr);
        }

        let mut res: f32 = 0.0;
        let ret = HFFaceComparison(&features[0], &features[1], &mut res);
        if ret != c_long(HSUCCEED as i64) {
            return Err("Unable to compare image".into());
        }

        dbg!(res);
    }

    Ok(())
}

fn main() {
    let first_image = CString::new("./img1.png").unwrap();
    let second_image = CString::new("./img1.png").unwrap();

    compare_image(vec![first_image, second_image]).unwrap();
}