use crate::processing::{worker::ImageWorker, job, job::WorkerJob};


pub struct Worker1;


pub enum Worker1Job {
    Resize(job::ResizeJob),
    Crop(job::CropJob)
}

impl TryFrom<WorkerJob> for Worker1Job {
    type Error = ();
    fn try_from(job: WorkerJob) -> Result<Self, ()> {
        match job {
            WorkerJob::Resize(job) => Ok(Worker1Job::Resize(job)),
            WorkerJob::Crop(job) => Ok(Worker1Job::Crop(job)),
            _ => Err(()),
        }
    }
}


impl ImageWorker for Worker1 {
    type WokerJob = Worker1Job;

    fn process(&mut self) {
        println!("Worker1::process()");
    }
}

