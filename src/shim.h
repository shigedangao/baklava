#pragma once
#include "inspireface.h"

// autocxx (built on cxx) cannot bind functions that take a `void**` argument,
// i.e. a pointer to an opaque `void*` handle. In InspireFace, HFImageBitmap,
// HFImageStream and HFSession are all `typedef void*`, so their out-parameters
// (PHFImageBitmap / PHFImageStream / PHFSession, all `void**`) make autocxx
// silently drop the wrapper for those functions.
//
// These thin inline wrappers return the handle through the return value
// instead, so the signature only ever uses a single-level `void*` plus an
// `HResult*` out-param, both of which autocxx binds cleanly.

inline HFImageBitmap baklava_create_image_bitmap_from_path(HPath filePath, HInt32 channels, HResult *result) {
    HFImageBitmap handle = nullptr;
    *result = HFCreateImageBitmapFromFilePath(filePath, channels, &handle);
    return handle;
}

inline HFImageStream baklava_create_image_stream_from_bitmap(HFImageBitmap bitmap, HFRotation rotation, HResult *result) {
    HFImageStream handle = nullptr;
    *result = HFCreateImageStreamFromImageBitmap(bitmap, rotation, &handle);
    return handle;
}

inline HFSession baklava_create_session_optional(HOption customOption, HFDetectMode detectMode, HInt32 maxDetectFaceNum,
                                                 HInt32 detectPixelLevel, HInt32 trackByDetectModeFPS, HResult *result) {
    HFSession handle = nullptr;
    *result = HFCreateInspireFaceSessionOptional(customOption, detectMode, maxDetectFaceNum, detectPixelLevel,
                                                 trackByDetectModeFPS, &handle);
    return handle;
}
