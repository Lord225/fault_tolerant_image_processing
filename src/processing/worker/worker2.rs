use crate::processing::{worker::ImageWorker, job, job::WorkerJob};


pub struct Worker2;

#[derive(Debug, Clone, Copy)]

pub enum Worker2Job {
    Blur(job::BlurJob),
    Brightness(job::BrightnessJob),
}

impl TryFrom<WorkerJob> for Worker2Job {
    type Error = ();
    fn try_from(job: WorkerJob) -> Result<Self, ()> {
        match job {
            WorkerJob::Brightness(job) => Ok(Worker2Job::Brightness(job)),
            WorkerJob::Blur(job) => Ok(Worker2Job::Blur(job)),
            _ => Err(()),
        }
    }
}


impl ImageWorker for Worker2 {
    type WokerJob = Worker2Job;

    fn process(&mut self) {
        println!("Worker1::process()");
    }
}

