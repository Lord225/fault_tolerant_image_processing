use crate::processing::{worker::ImageWorker, job};


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
    type WokerJob = Worker1Job;

    fn process(&mut self, job: job::Job) {
        println!("Worker1::process()");
    }
}

