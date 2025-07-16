use std::{error::Error, fmt};

/// FFIError is a list of possible error that the library can return.
#[derive(Debug)]
pub enum FFIError<'a> {
    ModelLoad,
    Session,
    Feature,
    MissingImage,
    Bitmap(&'a str),
    Stream(&'a str),
    FaceTrack(&'a str),
    Comparison(&'a str),
}

impl<'a> Error for FFIError<'a> {}

impl<'a> fmt::Display for FFIError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::ModelLoad => write!(f, "Unable to load the model"),
            Self::Session => write!(f, "Unable to load the session"),
            Self::Feature => write!(f, "Unable to create the feature"),
            Self::MissingImage => write!(f, "Unable to load the image"),
            Self::Bitmap(msg) => write!(f, "Unable to create a bitmap due to: {msg}"),
            Self::Stream(msg) => write!(f, "Unable to create stream image due to: {msg}"),
            Self::FaceTrack(msg) => write!(f, "Facetrack encountered an error due to: {msg}"),
            Self::Comparison(msg) => write!(f, "Unable to compare image due to: {msg}"),
        }
    }
}
