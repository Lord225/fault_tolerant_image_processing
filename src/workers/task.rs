use image::RgbImage;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Job {
    Resize{
        size: (u32, u32),
    },
    Crop{
        size: (u32, u32),
    },
    Blur{
        size: f32,
    },
    Brightness{
        size: f32,
    }
}


#[derive(Debug, Clone)]
pub struct Task {
    job: Job,
    data: RgbImage,
}
