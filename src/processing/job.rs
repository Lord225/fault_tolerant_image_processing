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
    Input,
}

impl JobType {
    pub fn new_resize(width: u32, height: u32) -> Self {
        JobType::Resize(ResizeJob(width, height))
    }

    pub fn new_blur(blur: f32) -> Self {
        JobType::Blur(BlurJob(blur))
    }

    pub fn input() -> Self {
        JobType::Input
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

impl<T> Job<T>
where
    T: TryFrom<JobType>,
{
    pub fn from_task(task: Task) -> Result<Self, Vec<i64>> {
        fn load_images_from_task_parents(task: &Task) -> Vec<Result<RgbImage, i64>> {
            task.parent_tasks
                .as_ref()
                .unwrap()
                .iter()
                .map(|task| {
                    task.data
                        .as_ref()
                        .ok_or(task.task_id)
                        .and_then(|data| load_image(data).map_err(|_| task.task_id))
                })
                .collect::<Vec<_>>()
        }

        let input = load_images_from_task_parents(&task);

        let task = match task.params.try_into() {
            Ok(task) => task,
            Err(_) => return Err(vec![]),
        };
        

        // if any is Err, return Err
        if input.iter().any(|result| result.is_err()) {
            Err(input
                .iter()
                .filter_map(|x| match x {
                    Ok(_) => None,
                    Err(id) => Some(*id),
                })
                .collect())
        } else {
            Ok(Self {
                task,
                data: input
                    .iter()
                    .map(|result| result.as_ref().unwrap().clone())
                    .collect::<Vec<_>>(),
            })
        }
    }
}
