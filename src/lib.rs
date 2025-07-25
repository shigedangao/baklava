use autocxx::c_void;
use autocxx::prelude::*;
use error::FFIError;
use ffi_wrapper::{
    HFCreateFaceFeature, HFCreateImageBitmapFromFilePath, HFCreateImageStreamFromImageBitmap,
    HFCreateInspireFaceSessionOptional, HFDetectMode, HFExecuteFaceTrack, HFFaceBasicToken,
    HFFaceComparison, HFFaceFeature, HFFaceFeatureWithRefExtractTo, HFGetTokens, HFImageBitmap,
    HFImageStream, HFLaunchInspireFace, HFMultipleFaceData, HFReleaseFaceFeature,
    HFReleaseImageBitmap, HFReleaseImageStream, HFReleaseInspireFaceSession, HFRotation, HFSession,
    HF_ENABLE_FACE_RECOGNITION, HSUCCEED,
};
use std::{
    ffi::CString,
    mem::{self},
};

pub mod error;
mod ffi_wrapper;

// Constants
const SUCCESS: i64 = HSUCCEED as i64;
const OUTPUT_MAX: f64 = 1.0;
const OUTPUT_MIN: f64 = 0.01;
const MIDDLE_SCORE: f64 = 0.6;
const STEEPNESS: f64 = 8.;
const RECOMMENDED_COSINE_THRESHOLD: f64 = 0.48;

/// InsightFace is a struct which handle the internal pointers to compare two faces and returns the cosine value
pub struct InsightFace {
    session: *mut c_void,
    features: Vec<HFFaceFeature>,
}

/// Methodology to use to compute get the cosine accross the selected image sources
pub enum Methodology {
    Mean,
    Median,
}

// Implement Send for InsightFace to allow it to be used in threads. Memory management should be safe...
// No need for Sync to be implemented as it's already done by the Mutex.
unsafe impl Send for InsightFace {}

impl InsightFace {
    /// Create a new InsightFace handler. It needs to be only call once as it build a model
    ///
    /// # Arguments
    ///
    /// * `model` - S
    /// * `sampling_size` - u8
    pub fn new<S: AsRef<str>>(
        model: S,
        sampling_size: u8,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let model = CString::new(model.as_ref())?;

        let session = HFSession::default();
        let mut session_ptr: *mut c_void = session as *mut c_void;

        // We only need to initialize the model once.
        unsafe {
            if HFLaunchInspireFace(model.as_ptr()).0 != SUCCESS {
                return Err(FFIError::ModelLoad.into());
            }
        }

        let features: Vec<HFFaceFeature> = (0..sampling_size + 1)
            .map(|_| unsafe { mem::zeroed() })
            .collect();

        unsafe {
            let res = HFCreateInspireFaceSessionOptional(
                c_int(HF_ENABLE_FACE_RECOGNITION as i32),
                HFDetectMode::HF_DETECT_MODE_ALWAYS_DETECT,
                c_int(1),
                c_int(-1),
                c_int(-1),
                &mut session_ptr,
            );

            if res.0 != SUCCESS {
                return Err(FFIError::Session.into());
            }
        }

        Ok(Self {
            session: session_ptr,
            features,
        })
    }

    /// Prepare a slice of a length of 2 images (faces) which will be compared
    ///
    /// # Arguments
    ///
    /// * `images` - &[S]
    pub fn prepare_images<S: AsRef<str> + std::clone::Clone>(
        &mut self,
        sources: &[S],
        target: S,
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        if sources.len() != self.features.len() - 1 {
            return Err(FFIError::SamplingSize.into());
        }

        unsafe {
            // Initialize an HFMultipleFaceData structure in the way c++ would do
            let mut multiple_face_data: HFMultipleFaceData = mem::zeroed();

            let mut images = sources.to_vec();
            images.push(target);

            for (idx, feature) in &mut self.features.iter_mut().enumerate() {
                if HFCreateFaceFeature(feature).0 != SUCCESS {
                    return Err(FFIError::Feature.into());
                }

                let img_path = images
                    .get(idx)
                    .map(|s| CString::new(s.as_ref()))
                    .ok_or(FFIError::MissingImage)??;

                let mut img_ptr = HFImageBitmap::default() as *mut c_void;
                let mut stream_ptr = HFImageStream::default() as *mut c_void;

                // Create bitmap from the file path. This will be used for face analysis
                match HFCreateImageBitmapFromFilePath(img_path.as_ptr(), c_int(3), &mut img_ptr).0 {
                    SUCCESS => {}
                    _ => {
                        return Err(
                            FFIError::Bitmap("image may not be the proper size or format").into(),
                        )
                    }
                }

                if img_ptr.is_null() {
                    return Err(FFIError::Bitmap("image bitmap pointer is null").into());
                }

                match HFCreateImageStreamFromImageBitmap(
                    img_ptr,
                    HFRotation::HF_CAMERA_ROTATION_0,
                    &mut stream_ptr,
                )
                .0
                {
                    SUCCESS => {}
                    _ => {
                        InsightFace::release_ptr(img_ptr, stream_ptr);
                        return Err(FFIError::Stream(
                            "Unable to create stream issue with rotation",
                        )
                        .into());
                    }
                }

                if stream_ptr.is_null() {
                    InsightFace::release_ptr(img_ptr, stream_ptr);
                    return Err(FFIError::Stream(
                        "Unable to create image from stream due to stream_ptr being null",
                    )
                    .into());
                }

                if HFExecuteFaceTrack(self.session, stream_ptr, &mut multiple_face_data).0
                    != SUCCESS
                {
                    InsightFace::release_ptr(img_ptr, stream_ptr);
                    return Err(FFIError::FaceTrack("").into());
                }

                let tokens_slice = HFGetTokens(&mut multiple_face_data);
                if tokens_slice.ptr.is_null() {
                    InsightFace::release_ptr(img_ptr, stream_ptr);
                    return Err(FFIError::FaceTrack(
                        "Unable to construct list of tokens due to tokens slice being null",
                    )
                    .into());
                }

                let tokens_ptr = tokens_slice.ptr as *mut HFFaceBasicToken;
                let tokens = std::slice::from_raw_parts_mut(tokens_ptr, tokens_slice.len as usize);

                let single_face = tokens.first_mut().ok_or_else(|| {
                    InsightFace::release_ptr(img_ptr, stream_ptr);
                    FFIError::FaceTrack("Unable to get the processed feature")
                })?;

                let res =
                    HFFaceFeatureWithRefExtractTo(self.session, stream_ptr, single_face, feature);
                if res.0 != SUCCESS {
                    InsightFace::release_ptr(img_ptr, stream_ptr);
                    return Err(
                        FFIError::FaceTrack("Unable to extract feature from stream_ptr").into(),
                    );
                }

                // Clean unused memory
                InsightFace::release_ptr(img_ptr, stream_ptr);
            }
        }

        Ok(self)
    }

    /// Compare the images and return the cosine similary which can range from 1 to -1
    ///
    /// # Arguments
    ///
    /// * `methodology` - Methodology
    pub fn compare_images(
        &self,
        methodology: Methodology,
    ) -> Result<(f32, f64), Box<dyn std::error::Error>> {
        let feature_target = self
            .features
            .last()
            .ok_or(FFIError::Comparison("Unable to get the second feature"))?;

        let mut cosine_result = Vec::new();

        for (idx, feature) in self.features.iter().enumerate() {
            // We want to skip the last value to compare
            if idx == self.features.len() - 1 {
                continue;
            }

            let mut res: f32 = 0.0;
            unsafe {
                let op_res = HFFaceComparison(feature, feature_target, &mut res);
                if op_res.0 != SUCCESS {
                    return Err(FFIError::Comparison("Comparison fail").into());
                }
            }

            cosine_result.push(res);
        }

        let cosine = match methodology {
            Methodology::Mean => {
                cosine_result.into_iter().fold(0., |acc, x| acc + x) / self.features.len() as f32
            }
            Methodology::Median => {
                // Sort the cosine result in ASC
                cosine_result.sort_unstable_by(|a, b| a.total_cmp(b));
                let mid = cosine_result.len() / 2;

                match cosine_result.len() % 2 == 0 {
                    true => {
                        let low = cosine_result
                            .get(mid - 1)
                            .ok_or(FFIError::Comparison("Unable to get the low mid"))?;

                        let high = cosine_result
                            .get(mid + 1)
                            .ok_or(FFIError::Comparison("Unable to get the high mid"))?;

                        (*low + *high) / 2.
                    }
                    false => cosine_result
                        .get(mid)
                        .copied()
                        .ok_or(FFIError::Comparison("Unable to get the median"))?,
                }
            }
        };

        // Compute the percentage as well by reusing the formula used in in InspireFace SDK
        Ok((cosine, Self::compute_percentage(cosine)))
    }

    /// Return whether the two faces are similar based on the cosine
    ///
    /// # Arguments
    ///
    /// * `cosine` - f32
    /// * `threshold` - Option<f64>
    pub fn is_similar(cosine: f32, threshold: Option<f64>) -> bool {
        cosine as f64 >= threshold.unwrap_or(RECOMMENDED_COSINE_THRESHOLD)
    }

    /// Compute the percentage of similarity from the computed cosine. Based on the inspireface SDK formula
    ///
    /// # Arguments
    ///
    /// * `cosine` - f32
    fn compute_percentage(cosine: f32) -> f64 {
        let bias = -f64::ln((OUTPUT_MAX - MIDDLE_SCORE) / (MIDDLE_SCORE - OUTPUT_MIN));
        let output_scale = OUTPUT_MAX - OUTPUT_MIN;

        let shifted_input = STEEPNESS * (cosine as f64 - RECOMMENDED_COSINE_THRESHOLD);
        let sigmoid = 1. / (1. + f64::exp(-shifted_input - bias));

        sigmoid * output_scale + OUTPUT_MIN
    }

    /// Release pointers which are used for the image & stream
    ///
    /// # Arguments
    ///
    /// * `img_ptr` - c_void
    /// * `stream_ptr` - c_void
    fn release_ptr(img_ptr: *mut c_void, stream_ptr: *mut c_void) {
        unsafe {
            HFReleaseImageBitmap(img_ptr);
            HFReleaseImageStream(stream_ptr);
        }
    }
}

impl Drop for InsightFace {
    fn drop(&mut self) {
        unsafe {
            HFReleaseInspireFaceSession(self.session);

            // Release all the features
            for feature in self.features.iter_mut() {
                // We can safely release the feature as it was created by us using mem::zeroed()
                HFReleaseFaceFeature(feature);
            }
        }
    }
}
