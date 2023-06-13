use image::RgbImage;
use log::{debug};

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

    fn process(&mut self, job: job::Job<Self::WorkerJob>) -> Result<RgbImage, ()> {
        debug!("Worker1::process()");

        match job {
            job::Job { task: Worker2Job::Brightness(_params), data } => {
                debug!("Brightness {:?}", _params);
                
                let img = data.first().unwrap();

                Ok(image::imageops::brighten(img, 
                                          _params.0 as i32,))
            },
            job::Job { task: Worker2Job::Blur(_params), mut data } => {
                debug!("Blur {:?}", _params);

                // get the first image as &mut 
                let img = data.first_mut().unwrap(); 
                
                Ok(
                    image::imageops::blur(img, _params.0)
                )
            },
        }
    }
}

impl Worker2 {
    pub fn new() -> Self {
        Worker2
    }
}
