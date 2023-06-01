pub mod worker1;
pub mod worker2;

use std::sync::mpsc::{self, Sender};
use image::RgbImage;

use crate::{database::{common::Database, repositories::task::Task}, processing::data_loader::save_image};

use super::job::{Job, JobType};

pub trait ImageWorker {
    type WorkerJob: TryFrom<JobType> + Send;

    fn process(&mut self, job: Job<Self::WorkerJob>) -> Result<RgbImage, ()>;
}

pub struct WorkerThread<Worker: ImageWorker+Send> {
    thread: Option<(std::thread::JoinHandle<()>, mpsc::Sender<Task>)>,
    phantom: std::marker::PhantomData<Worker>,
}

impl<Worker: ImageWorker+Send+'static> WorkerThread<Worker> {
    pub fn new() -> Self {
        Self {
            thread: None,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn start(&mut self, worker: Worker, journal: Database) {
        let (tx, rx) = mpsc::channel();

        let thread = std::thread::spawn(move || {
            Self::thread_body(worker, journal, rx);
        });

        self.thread = Some((thread, tx));
    }

    pub fn send_task(&mut self, task: Task) {
        if let Some((_, tx)) = &self.thread {
            tx.send(task).unwrap();
        } else {
            println!("WorkerThread::send_task(): Thread is not running.. Skiping task");
        }
    }

    pub fn restore_thread<F>(&mut self, f: F)
    where F: FnOnce() -> (Worker, Database)
     {
        // check if thread is alive
        if let Some((t, tx)) = &self.thread {
            if t.is_finished() {
                println!("WorkerThread::restore_thread(): Thread died. restoring");
                let (worker, journal) = f();
                let (tx, rx) = mpsc::channel();

                let thread = std::thread::spawn(move || {
                    Self::thread_body(worker, journal, rx);
                });

                self.thread = Some((thread, tx));
            }
        }
    } 

    fn thread_body(mut worker: Worker, mut journal: Database, channel: mpsc::Receiver<Task>) {
        loop {
            let task = channel.recv().unwrap();
            let task_id = task.task_id;

            let result = match Job::<Worker::WorkerJob>::from_task(task) {
                Ok(job) => {
                    println!("WorkerThread::thread_body(): Received task: {}", task_id);

                    match  worker.process(job) {
                        Ok(image) => {
                            println!("WorkerThread::thread_body(): Job processed successfully");

                            save_image(&image).unwrap();

                            Ok(())
                        },
                        Err(_) => {
                            println!("WorkerThread::thread_body(): Error processing job");

                            Err(())
                        },
                    }
                },
                Err(failed_tasks_ids) => {
                    println!("WorkerThread::thread_body(): Parent was marked as completed, but failed: {:?}", failed_tasks_ids);

                    for failed_task_id in failed_tasks_ids {
                        journal.mark_task_as_failed(failed_task_id).unwrap();
                    }

                    Err(())
                },
            };

            match result {
                Ok(()) => journal.mark_task_as_completed(task_id).unwrap(),
                Err(()) => journal.mark_task_as_failed(task_id).unwrap(),
            }           

            // sleep
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}