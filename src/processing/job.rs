use serde::{Serialize, Deserialize};

#[derive(Debug, Clone,Copy, Serialize, Deserialize)]
pub struct ResizeJob(pub u32,pub u32);
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CropJob(pub u32,pub u32);
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BlurJob(pub f32);
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BrightnessJob(pub f32);


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WorkerJob{
    Resize(ResizeJob),
    Crop(CropJob),
    Blur(BlurJob),
    Brightness(BrightnessJob),
}
