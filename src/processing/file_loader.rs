



use std::fmt::Formatter;

use image::{RgbImage, ImageError};


#[derive(Debug, Clone, Copy)]
pub struct DataLoaderError;

impl std::fmt::Display for DataLoaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DataLoaderError")
    }
}

impl From<ImageError> for DataLoaderError {
    fn from(_: ImageError) -> Self {
        DataLoaderError
    }
}

impl std::error::Error for DataLoaderError {}

pub fn load_image(path: &str) -> Result<RgbImage, DataLoaderError> {
    Ok(image::open(path)?.to_rgb8())
}

pub fn save_image(path: &str, image: &RgbImage) -> Result<(), DataLoaderError> {
    image.save(path)?;
    Ok(())
}
