pub mod worker1;
pub mod worker2;

use image::RgbImage;
use log::{error, info, warn};
use std::{
    marker::PhantomData,
    sync::{
        mpsc::{self}
    },
    time::Duration,
};

use uuid::Uuid;

use crate::{
    database::{
        common::{Database, ErrorType},
        repositories::task::Task,
    },
    processing::data_loader::save_image_with_path,
    temp::from_temp, engine::ConfigType,
};

use super::job::{Job, JobType};

#[derive(Debug, Clone, Copy)]
pub struct WorkerErrorConfig {
    pub throttle: Duration,
    pub random_error_chance: f32,
    pub random_not_save_chance: f32,
    pub paused: bool,
}

impl Default for WorkerErrorConfig{
    fn default() -> Self {
        Self { throttle: Duration::from_millis(200), random_error_chance: 0.0, random_not_save_chance: 0.0, paused: true }
    }
}

pub trait ImageWorker {
    type WorkerJob: TryFrom<JobType> + Send;

    fn process(&mut self, job: Job<Self::WorkerJob>) -> Result<RgbImage, ()>;
}

pub struct WorkerThread<Worker: ImageWorker + Send> {
    thread: Option<(
        std::thread::JoinHandle<()>,
        mpsc::Sender<Task>,
    )>,
    phantom: PhantomData<Worker>,
}

impl<Worker: ImageWorker + Send + 'static> WorkerThread<Worker> {
    pub fn new() -> Self {
        Self {
            thread: None,

            phantom: PhantomData,
        }
    }

    pub fn start(&mut self, worker: Worker, journal: Database, config: ConfigType) {
        let (tx, rx) = mpsc::channel();  
        let thread = std::thread::spawn(move || {
            Self::thread_body(worker, journal, rx, config);
        });        

        self.thread = Some((thread, tx));
    }

    pub fn send_task(&mut self, task: Task) -> Result<(), ErrorType> {
        if let Some((_, tx)) = &self.thread {
            match tx.send(task) {
                Ok(_) => Ok(()),
                Err(_) => Err(ErrorType::WorkerThreadFailed),
            }
        } else {
            Err(ErrorType::WorkerThreadFailed)
        }
    }

    pub fn thread_died(&self) -> bool {
        if let Some((t, _)) = &self.thread {
            t.is_finished()
        } else {
            true
        }
    }

    pub fn restore_thread<F>(&mut self, f: F)
    where
        F: FnOnce() -> (Worker, Database, ConfigType),
    {
        // check if thread is alive
        if self.thread.is_none() {
            let (worker, journal, config) = f();
            info!("Starting worker thread");
            self.start(worker, journal, config);
        } else if self.thread_died() && self.thread_died() {
            let (worker, journal, config) = f();
            warn!("Thread died. Restarting...");
            self.start(worker, journal, config);
        }
    }

    fn thread_body(
        mut worker: Worker,
        mut journal: Database,
        channel: mpsc::Receiver<Task>,
        config: ConfigType,
    ) {
        loop {
            let task = channel.recv().unwrap();
            let task_id = task.task_id;
            let filename = from_temp(&format!("{}.bmp", Uuid::new_v4()));

            if config.read().unwrap().paused {
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }

            let result = match Job::<Worker::WorkerJob>::from_task(task) {
                Ok(job) => {
                    info!("Received task: {}", task_id);

                    // fail randomly with chance
                    if rand::random::<f32>() < config.read().unwrap().random_error_chance {
                        error!("Random error");
                        panic!("random error");
                    }

                    match worker.process(job) {
                        Ok(image) => {
                            info!("Job processed successfully");

                            // save_image(&image).unwrap();
                            save_image_with_path(&filename, &image).unwrap();

                            Ok(())
                        }
                        Err(_) => {
                            warn!("Error processing job");

                            Err(())
                        }
                    }
                }
                Err(failed_tasks_ids) => {
                    warn!("Parents were marked as completed, but were not found in the database, ids: {:?}", failed_tasks_ids);
                    for failed_task_id in failed_tasks_ids {
                        journal.mark_task_as_failed(failed_task_id).unwrap();
                    }

                    Err(())
                }
            };

            match result {
                Ok(()) => journal.mark_task_as_completed(task_id, &filename).unwrap(),
                Err(()) => journal.mark_task_as_failed(task_id).unwrap(),
            }

            // sleep
            std::thread::sleep(config.read().unwrap().throttle);
        }
    }
}
