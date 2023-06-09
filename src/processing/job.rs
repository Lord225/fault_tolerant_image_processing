use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResizeJob(pub u32, pub u32);
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CropJob(pub u32, pub u32, pub u32, pub u32);
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BlurJob(pub f32);
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BrightnessJob(pub f32);
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OverlayJob(pub u32, pub u32);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum JobType {
    Resize(ResizeJob),
    Crop(CropJob),
    Blur(BlurJob),
    Brightness(BrightnessJob),
    Overlay(OverlayJob),
    Input,
}

impl JobType {
    #[allow(dead_code)]
    pub fn new_resize(width: u32, height: u32) -> Self {
        JobType::Resize(ResizeJob(width, height))
    }
    #[allow(dead_code)]
    pub fn new_blur(blur: f32) -> Self {
        JobType::Blur(BlurJob(blur))
    }
    #[allow(dead_code)]
    pub fn new_brightness(brightness: f32) -> Self {
        JobType::Brightness(BrightnessJob(brightness))
    }
    #[allow(dead_code)]
    pub fn new_overlay(x: u32, y: u32) -> Self {
        JobType::Overlay(OverlayJob(x, y))
    }

    #[allow(dead_code)]
    pub fn new_crop(x: u32, y: u32, width: u32, height: u32) -> Self {
        JobType::Crop(CropJob(x, y, width, height))
    }
    #[allow(dead_code)]
    pub fn input() -> Self {
        JobType::Input
    }

    pub fn input_count(&self) -> usize {
        match self {
            JobType::Resize(_) => 1,
            JobType::Crop(_) => 1,
            JobType::Blur(_) => 1,
            JobType::Brightness(_) => 1,
            JobType::Overlay(_) => 2,
            JobType::Input => 0,
        }
    }
}

use crate::{database::repositories::task::Task, processing::data_loader::load_image};
use image::RgbImage;

#[derive(Debug)]
pub struct Job<T>
where
    T: TryFrom<JobType>,
{
    pub task: T,
    pub data: Vec<RgbImage>,
}

type InvalidTask = i64;
type LoadDataResult = Result<RgbImage, InvalidTask>;

impl<T> Job<T>
where
    T: TryFrom<JobType>,
{
    pub fn from_task(task: Task) -> Result<Self, Vec<InvalidTask>> {
        fn load_images_from_task_parents(parents: &[Task]) -> Vec<LoadDataResult> {
            parents
                .iter()
                .map(|task| {
                    task.data
                        .as_ref()
                        .ok_or(task.task_id)
                        .and_then(|data| load_image(data).map_err(|_| task.task_id))
                })
                .collect::<Vec<_>>()
        }

        fn collect_errors(inputs: Vec<LoadDataResult>) -> Vec<InvalidTask> {
            inputs
                .iter()
                .filter_map(|x| match x {
                    Ok(_) => None,
                    Err(id) => Some(*id),
                })
                .collect()
        }
        fn unwrap_inputs(inputs: Vec<LoadDataResult>) -> Vec<RgbImage> {
            inputs
                .iter()
                .map(|result| result.as_ref().unwrap().clone())
                .collect::<Vec<_>>()
        }

        let input: _ = load_images_from_task_parents(&task.parent_tasks.unwrap());

        let task = match task.params.try_into() {
            Ok(task) => task,
            Err(_) => return Err(vec![]),
        };

        if input.iter().any(|result| result.is_err()) {
            Err(collect_errors(input))
        } else {
            Ok(Self {
                task,
                data: unwrap_inputs(input),
            })
        }
    }
}
