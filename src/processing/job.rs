use serde::{Serialize, Deserialize};

#[derive(Debug, Clone,Copy, Serialize, Deserialize)]
pub struct ResizeJob(u32, u32);
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CropJob(u32, u32);
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BlurJob(f32);
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BrightnessJob(f32);


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum JobType{
    Resize(ResizeJob),
    Crop(CropJob),
    Blur(BlurJob),
    Brightness(BrightnessJob),
}


impl JobType {
    pub fn new_resize(width:u32, height:u32) -> Self {
        JobType::Resize(ResizeJob(width,height))
    }
    pub fn new_blur(blur:f32) -> Self {
        JobType::Blur(BlurJob(blur))
    }
}

use image::RgbImage;

#[derive(Debug, Clone)]
pub struct Job {
    job: JobType,
    data: Vec<RgbImage>,
}
