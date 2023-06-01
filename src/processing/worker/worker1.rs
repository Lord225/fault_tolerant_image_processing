use image::RgbImage;
use image;

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
                dbg!("Worker1::process() Resize");
                
                let img = data.first().unwrap();

                Ok(image::imageops::resize(img, 
                                          _params.0, 
                                         _params.1, 
                                          image::imageops::FilterType::Nearest))
            },
            job::Job { task: Worker1Job::Crop(_params), mut data } => {
                dbg!("Worker1::process() Crop");

                // get the first image as &mut 
                let img = data.first_mut().unwrap(); 
                
                Ok(
                    image::imageops::crop(img, 
                                          _params.0, 
                                         _params.1, 
                                          _params.2, 
                                         _params.3).to_image()
                )
            },
        }
    }
}

impl Worker1 {
    pub fn new() -> Self {
        Worker1
    }
}