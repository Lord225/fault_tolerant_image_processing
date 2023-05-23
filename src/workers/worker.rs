use super::task::Task;
use std::sync::mpsc;
use crate::journal::Journal;

// this is worker mod,
// it contains an worker that 
// will take data from channel
// and process it
// it is fault tolerant so it will save progress
// into the database

pub trait ImageWorker {
    fn process(&mut self);
}

struct WorkerWrapper<Worker: ImageWorker> {
    channel: mpsc::Receiver<Task>,
    journal: Journal,
    worker: Worker,
}

impl<Worker: ImageWorker> WorkerWrapper<Worker> {
    fn new(channel: mpsc::Receiver<Task>, journal: Journal, worker: Worker) -> Self {
        Self {
            channel,
            journal,
            worker,
        }
    }

    fn process(&mut self) {
        loop {
            let task = self.channel.recv().unwrap();

            
            
        }

    }
}