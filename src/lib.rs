//! Baklava is a rust wrapper around the InsightFace library to perform face comparison
//! It returns a cosine and percentage similarity between the given image's face against a target face.
//!
//! The cosine similarity could be either positive or negative, depending on the orientation of the faces and how close they are to each other.
//! Usually, a cosine similarity of 0.7 or higher is considered a good match.
//!
//! The library can be used as simple as the example below.
//!
//! ```
//! use baklava::{InsightFace, Methodology};
//!
//! let (cosine, percentage) = InsightFace::new("./Megatron", Some(3))
//!    .unwrap()
//!    .prepare_images(&[
//!        "./face1_test.png",
//!        "./face2_test.png",
//!    ]).unwrap()
//!    .prepare_target_image("./face1_test.png").unwrap()
//!    .compare_images(Methodology::Mean).unwrap();
//! ```
//!
//! To perform the comparison baklava required you to downlaod a model from the InsightFace repository
//! that can be found at this link: <https://github.com/HyperInspire/InspireFace?tab=readme-ov-file#resource-package-list>
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
use std::str::FromStr;
use std::sync::Mutex;
use std::{
    ffi::CString,
    mem::{self},
    sync::Arc,
    thread,
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
    src_features: Vec<HFFaceFeature>,
    target_feature: HFFaceFeature,
    chunks: Option<usize>,
}

/// SessionHandler is a wrapper around the session pointer in order to be able to be used in the context of multithreading
struct SessionHandler {
    session: *mut c_void,
}

/// Methodology to use to compute get the cosine accross the selected image sources
#[derive(PartialEq, Eq)]
pub enum Methodology {
    /// Perform a mean calculation over the cosine values
    Mean,
    /// Perform a median calculation over the cosine values
    Median,
}

// Implement Send for InsightFace to allow it to be used in threads. Memory management should be safe...
// No need for Sync to be implemented as it's already done by the Mutex.
unsafe impl Send for InsightFace {}
unsafe impl Send for SessionHandler {}
unsafe impl Send for HFFaceFeature {}

// Add Sync implementation for InsightFace should the use want to use it within a LazyLock or OnceLock
unsafe impl Sync for InsightFace {}

impl InsightFace {
    /// Create a new InsightFace handler. It needs to be only call once as it build a model. Therefore it's recommended
    /// to use it within an `Arc<Mutex<InsightFace>>` to ensure thread safety. Chunk size is used in order to split the image data into smaller chunks for processing.
    ///
    /// # Arguments
    ///
    /// * `model` - S
    /// * `chunk_size` - u8
    ///
    ///
    /// # Examples
    /// ```
    /// use std::sync::Arc;
    /// use std::sync::Mutex;
    /// use baklava::InsightFace;
    ///
    /// let insight_face = Arc::new(Mutex::new(InsightFace::new("./Megatron", None).unwrap()));
    /// ```
    pub fn new<S: AsRef<str>>(
        model: S,
        chunk_size: Option<usize>,
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
            src_features: vec![],
            target_feature: unsafe { mem::zeroed() },
            chunks: chunk_size,
        })
    }

    /// Prepare a slice of a images to be compared toward the target
    ///
    /// # Arguments
    ///
    /// * `sources` - `&[S]`
    pub fn prepare_images<S: AsRef<str> + std::clone::Clone + Send + Sync + Copy>(
        &mut self,
        sources: &[S],
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        self.src_features = (0..sources.len())
            .map(|_| unsafe { mem::zeroed() })
            .collect();

        let images = sources.to_vec();
        let images_arc_handle = Arc::new(images);

        let send_session = Arc::new(Mutex::new(SessionHandler {
            session: self.session,
        }));

        // By default we're going to spawn 1 thread which will do the task
        // We'll use the chunk_size as a reference to which the baklava library will spawn a set of threads.
        let chunk_size = self.chunks.unwrap_or(1);

        // Split the list of features in a set of chunks
        let chunks_features = self.src_features.chunks_mut(chunk_size).collect::<Vec<_>>();
        let chunks_len = chunks_features.len();

        let chf = Arc::new(Mutex::new(chunks_features));

        thread::scope(|s| -> Result<(), Box<dyn std::error::Error>> {
            let img_handle = images_arc_handle.clone();
            let session_incr = send_session.clone();

            // We avoid getting the feature here as the *mut HFFaceFeature is not Send.
            for idx in 0..chunks_len {
                let images_clone = img_handle.clone();
                // Increase ref conting
                let session_incr = session_incr.clone();

                // increase ref counting of chunks features
                let chf = chf.clone();

                s.spawn(move || -> Result<(), FFIError> {
                    let images_clone = images_clone.clone();
                    let session_incr = session_incr.clone();

                    // As we're now spawning a new thread increase again the ref counting
                    let chf = chf.clone();

                    // Acquire the mutex
                    let mut mutex = chf
                        .lock()
                        .map_err(|_| FFIError::IO("Unable to acquire lock"))?;

                    let chunk = mutex
                        .get_mut(idx)
                        .ok_or(FFIError::IO("Unable to acquire lock"))?;
                    let c = chunk;

                    for (iidx, feature) in c.iter_mut().enumerate() {
                        let mut counter = iidx;
                        if idx > 0 {
                            // Increase the counter based on the current position and the chunk_size. This ensure that each thread get it's own photo to process.
                            counter = idx * chunk_size + iidx;
                        }

                        let img_path = images_clone
                            .clone()
                            .get(counter)
                            .map(|s| CString::new(s.as_ref()))
                            .ok_or(FFIError::MissingImage)?
                            .map_err(|_| FFIError::MissingImage)?;

                        InsightFace::prepare_image_for_comparison(
                            feature,
                            img_path.clone(),
                            session_incr.clone(),
                        )
                        .map_err(|_| FFIError::IO("Unable to prepare image"))?;
                    }

                    Ok(())
                });
            }

            Ok(())
        })?;

        Ok(self)
    }

    /// Prepare the target image that will be compared against the sources images
    ///
    /// # Arguments
    ///
    /// * `target_img_path` - S
    pub fn prepare_target_image<S: AsRef<str>>(
        &mut self,
        target_img_path: S,
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let send_session = Arc::new(Mutex::new(SessionHandler {
            session: self.session,
        }));

        let img_path =
            CString::from_str(target_img_path.as_ref()).map_err(|_| FFIError::Feature)?;

        InsightFace::prepare_image_for_comparison(
            &mut self.target_feature,
            img_path,
            send_session,
        )?;

        Ok(self)
    }

    /// Prepare a set of images for comparison
    ///
    /// # Arguments
    ///
    /// * `feature` - *mut HFFaceFeature
    /// * `img_path` - CString
    /// * `session_handler` - Arc<Mutex<SessionHandler>>
    fn prepare_image_for_comparison(
        feature: *mut HFFaceFeature,
        img_path: CString,
        session_handler: Arc<Mutex<SessionHandler>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            // Initialize an HFMultipleFaceData structure in the way c++ would do
            let mut multiple_face_data: HFMultipleFaceData = mem::zeroed();

            if HFCreateFaceFeature(feature).0 != SUCCESS {
                return Err(FFIError::Feature.into());
            }

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
                    return Err(
                        FFIError::Stream("Unable to create stream issue with rotation").into(),
                    );
                }
            }

            if stream_ptr.is_null() {
                InsightFace::release_ptr(img_ptr, stream_ptr);
                return Err(FFIError::Stream(
                    "Unable to create image from stream due to stream_ptr being null",
                )
                .into());
            }

            let mutex = session_handler
                .lock()
                .map_err(|_| FFIError::Comparison("Unable to acquire the session handler lock"))?;

            if HFExecuteFaceTrack(mutex.session, stream_ptr, &mut multiple_face_data).0 != SUCCESS {
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
                HFFaceFeatureWithRefExtractTo(mutex.session, stream_ptr, single_face, feature);
            if res.0 != SUCCESS {
                InsightFace::release_ptr(img_ptr, stream_ptr);
                return Err(
                    FFIError::FaceTrack("Unable to extract feature from stream_ptr").into(),
                );
            }

            // Clean unused memory
            InsightFace::release_ptr(img_ptr, stream_ptr);
        }

        Ok(())
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
        let mut cosine_result = Vec::new();

        for feature in self.src_features.iter() {
            let mut res: f32 = 0.0;
            unsafe {
                let op_res = HFFaceComparison(feature, &self.target_feature, &mut res);
                if op_res.0 != SUCCESS {
                    return Err(FFIError::Comparison("Comparison fail").into());
                }
            }

            cosine_result.push(res);
        }

        // When the sample size is too small. We're unable to perform the median methodology. Hence better use the mean methodology in that case
        if cosine_result.len() == 2 && methodology == Methodology::Median {
            return Err(FFIError::Comparison(
                "Sample size is too small. You should consider to use the mean methodology instead",
            )
            .into());
        }

        let cosine = match methodology {
            Methodology::Mean => {
                cosine_result.into_iter().fold(0., |acc, x| acc + x)
                    / self.src_features.len() as f32
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
    /// * `threshold` - `Option<f64>`
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
            for feature in self.src_features.iter_mut() {
                // We can safely release the feature as it was created by us using mem::zeroed()
                HFReleaseFaceFeature(feature);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{InsightFace, Methodology};
    use reqwest::blocking::Client;
    use std::sync::{Arc, LazyLock, Mutex};

    static INSIGHT_FACE_CLIENT: LazyLock<Arc<Mutex<InsightFace>>> = LazyLock::new(|| {
        download_megatron_source();

        Arc::new(Mutex::new(InsightFace::new("./Megatron", None).unwrap()))
    });

    fn download_megatron_source() {
        let client = Client::new();
        let response = client
            .get("https://github.com/HyperInspire/InspireFace/releases/download/v1.x/Megatron")
            .send()
            .unwrap();
        let bytes = response.bytes().unwrap();
        std::fs::write("Megatron", bytes).unwrap();
    }

    #[test]
    fn expect_to_compare_image() {
        let mut model = INSIGHT_FACE_CLIENT.lock().unwrap();

        // Compare two images
        let prep_image_set_1 = model
            .prepare_images(&["./face1_test.png", "./face2_test.png"])
            .unwrap()
            .prepare_target_image("./face1_test.png");

        assert!(prep_image_set_1.is_ok());

        let prep_image_set_1 = prep_image_set_1.unwrap();
        let (cos, percentage) = prep_image_set_1.compare_images(Methodology::Mean).unwrap();

        assert!(cos > 0.6);
        assert!(percentage > 0.6);
    }

    #[test]
    fn expect_to_compare_image_with_median_methodology() {
        let mut model = INSIGHT_FACE_CLIENT.lock().unwrap();

        // Compare two images
        let prep_image_set_1 = model
            .prepare_images(&["./face1_test.png", "./face2_test.png", "./face1_test.png"])
            .unwrap()
            .prepare_target_image("./face1_test.png");

        assert!(prep_image_set_1.is_ok());

        let prep_image_set_1 = prep_image_set_1.unwrap();
        let (cos, percentage) = prep_image_set_1
            .compare_images(Methodology::Median)
            .unwrap();

        assert!(cos > 0.6);
        assert!(percentage > 0.6);
    }

    #[test]
    fn expect_median_methodology_to_fail() {
        let mut model = INSIGHT_FACE_CLIENT.lock().unwrap();

        // Compare two images
        let prep_image_set_1 = model
            .prepare_images(&["./face1_test.png", "./face2_test.png"])
            .unwrap()
            .prepare_target_image("./face1_test.png");

        assert!(prep_image_set_1.is_ok());

        let prep_image_set_1 = prep_image_set_1.unwrap();
        let res = prep_image_set_1.compare_images(Methodology::Median);

        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Unable to compare image due to: Sample size is too small. You should consider to use the mean methodology instead"
        );
    }
}
