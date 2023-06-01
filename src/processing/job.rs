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

type InvalidTask = i64;
type LoadDataResult = Result<RgbImage, InvalidTask>;

impl<T> Job<T>
where
    T: TryFrom<JobType>,
{
    pub fn from_task(task: Task) -> Result<Self, Vec<InvalidTask>> {
        fn load_images_from_task_parents(parents: &Vec<Task>) -> Vec<LoadDataResult> {
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
                task: task,
                data: unwrap_inputs(input),
            })
        }
    }
}
