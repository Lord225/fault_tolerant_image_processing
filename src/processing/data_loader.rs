use std::fmt::Formatter;
use uuid::Uuid;
use image::{RgbImage, ImageError};
use lazy_static::lazy_static;
use std::env::var;

lazy_static! {
    static ref IMAGE_PATH: String = var("IMG_PATH").unwrap();
}


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
    dbg!(path);
    Ok(image::open(path)?.to_rgb8())
}

pub fn save_image_with_path(path: &str, image: &RgbImage) -> Result<(), DataLoaderError> {
    image.save(path)?;
    Ok(())
}

pub fn save_image(image: &RgbImage) -> Result<(), DataLoaderError> {
    save_image_with_path(&format!("{}/{}.bmp", *IMAGE_PATH, Uuid::new_v4()), image)
}