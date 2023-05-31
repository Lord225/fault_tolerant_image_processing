use image::RgbImage;

use crate::{processing::{worker::ImageWorker, job::{self}}};


pub struct Worker1;

#[derive(Debug, Clone, Copy)]

pub enum Worker1Job {
    Resize(job::ResizeJob),
    Crop(job::CropJob)
}

impl TryFrom<job::JobType> for Worker1Job {
    type Error = ();
    fn try_from(job: job::JobType) -> Result<Self, ()> {
        match job {
            job::JobType::Resize(job) => Ok(Worker1Job::Resize(job)),
            job::JobType::Crop(job) => Ok(Worker1Job::Crop(job)),
            _ => Err(()),
        }
    }
}


impl ImageWorker for Worker1 {
    type WorkerJob = Worker1Job;

    fn process(&mut self, job: job::Job<Self::WorkerJob>) -> Result<RgbImage, ()> {
        match job {
            job::Job { task: Worker1Job::Resize(_params), data } => {
                println!("Worker1::process() Resize");
                
                Ok(data[0].clone())
            },
            job::Job { task: Worker1Job::Crop(_params), data } => {
                println!("Worker1::process() Crop");
                
                Ok(data[0].clone())
            },
        }
    }
}

impl Worker1 {
    pub fn new() -> Self {
        Worker1
    }
}