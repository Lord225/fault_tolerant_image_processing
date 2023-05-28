use image::RgbImage;

use super::job::WorkerJob;



#[derive(Debug, Clone)]
pub struct Task {
    job: WorkerJob,
    data: RgbImage,
}
