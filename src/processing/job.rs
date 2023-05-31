use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResizeJob(u32, u32);
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CropJob(u32, u32);
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BlurJob(f32);
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BrightnessJob(f32);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum JobType {
    Resize(ResizeJob),
    Crop(CropJob),
    Blur(BlurJob),
    Brightness(BrightnessJob),
}

impl JobType {
    pub fn new_resize(width: u32, height: u32) -> Self {
        JobType::Resize(ResizeJob(width, height))
    }

    pub fn new_blur(blur: f32) -> Self {
        JobType::Blur(BlurJob(blur))
    }
}

use crate::database::repositories::task::Task;
use image::RgbImage;

use super::file_loader::{load_image, DataLoaderError};

#[derive(Debug)]
pub struct Job {
    job: Task,
    data: Vec<RgbImage>,
}

impl Job {
    pub fn from_task(task: Task) -> Result<Self, DataLoaderError> {
        let input = task
            .parent_tasks
            .as_ref()
            .ok_or(DataLoaderError)?
            .iter()
            .map(|task| &task.data)
            .map(|path| load_image(path))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            job: task,
            data: input,
        })
    }
}
