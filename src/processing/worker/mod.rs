pub mod worker1;
pub mod worker2;

use std::sync::mpsc::{self, Sender};
use image::RgbImage;

use crate::database::{common::Database, repositories::task::Task};

use super::job::Job;
pub trait ImageWorker {
    type WokerJob;
    fn process(&mut self, job: Job) -> Result<RgbImage, ()>;
}

struct WorkerThread<Worker: ImageWorker+Send> {
    thread: Option<std::thread::JoinHandle<()>>,
    phantom: std::marker::PhantomData<Worker>,
}

impl<Worker: ImageWorker+Send+'static> WorkerThread<Worker> {
    pub fn new() -> Self {
        Self {
            thread: None,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn start(&mut self, worker: Worker, journal: Database) -> Sender<Task> {
        let (tx, rx) = mpsc::channel();

        let thread = std::thread::spawn(move || {
            Self::thread_body(worker, journal, rx);
        });

        self.thread = Some(thread);

        tx
    }

    fn thread_body(mut worker: Worker, mut journal: Database, channel: mpsc::Receiver<Task>) {
        loop {
            let task = channel.recv().unwrap();
            let job = Job::from_task(task).unwrap();
            
            let result = worker.process(job);

            match result {
                Ok(image) => {
                    
                },
                Err(_) => {
                    println!("WorkerThread::thread_body(): Error processing job");
                },
            }

            // sleep
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}