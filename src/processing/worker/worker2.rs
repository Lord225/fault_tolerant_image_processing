use image::RgbImage;
use log::{info, debug};

use crate::processing::{worker::ImageWorker, job};


pub struct Worker2;

#[derive(Debug, Clone, Copy)]
pub enum Worker2Job {
    Blur(job::BlurJob),
    Brightness(job::BrightnessJob),
}

impl TryFrom<job::JobType> for Worker2Job {
    type Error = ();
    fn try_from(job: job::JobType) -> Result<Self, ()> {
        match job {
            job::JobType::Brightness(job) => Ok(Worker2Job::Brightness(job)),
            job::JobType::Blur(job) => Ok(Worker2Job::Blur(job)),
            _ => Err(()),
        }
    }
}


impl ImageWorker for Worker2 {
    type WorkerJob = Worker2Job;

    fn process(&mut self, _job: job::Job<Self::WorkerJob>) -> Result<RgbImage, ()> {
        debug!("Worker1::process()");

        todo!()
    }
}

impl Worker2 {
    pub fn new() -> Self {
        Worker2
    }
}
