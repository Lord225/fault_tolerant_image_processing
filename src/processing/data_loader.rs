use std::fmt::Formatter;
use log::trace;
use uuid::Uuid;
use image::{RgbImage, ImageError};

use crate::temp::from_temp;


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
    trace!("loading {}", path);
    Ok(image::open(path)?.to_rgb8())
}

pub fn save_image_with_path(path: &str, image: &RgbImage) -> Result<(), DataLoaderError> {
    trace!("saving {}", path);
    image.save(path)?;
    Ok(())
}

pub fn save_image(image: &RgbImage) -> Result<(), DataLoaderError> {
    save_image_with_path(&from_temp(&format!("{}.bmp", Uuid::new_v4())), image)
}