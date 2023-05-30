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
pub enum WorkerJob{
    Resize(ResizeJob),
    Crop(CropJob),
    Blur(BlurJob),
    Brightness(BrightnessJob),
}


impl WorkerJob {
    pub fn new_resize(width:u32,height:u32 ) -> Self {
        WorkerJob::Resize(ResizeJob(width,height))
    }
    pub fn new_blur(blur:f32) -> Self {
        WorkerJob::Blur(BlurJob(blur))
    }

    // TODO more constructors
}