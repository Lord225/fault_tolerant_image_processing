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
    type WokerJob = Worker2Job;

    fn process(&mut self, job: job::Job) {
        println!("Worker1::process()");
    }
}

