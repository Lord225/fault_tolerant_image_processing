use log::{info, error, warn};
use no_panic::no_panic;
use std::thread;
use std::time::Duration;
use crate::{processing::worker::{worker1::{Worker1, Worker1Job}, worker2::{Worker2, Worker2Job}, WorkerThread}, database::common::{try_open_connection, Database, ErrorType}};

struct Engine {
    worker1: WorkerThread<Worker1>,
    worker2: WorkerThread<Worker2>,
}

enum EngineState {
    WorkDone,
    Idle,
}

fn schedule_thread_body() -> !
{    
    
    fn check_if_workers_are_workin(engine: &mut Engine) {
        engine.start_failed_workers();
    }
    fn claim_tasks(db: &mut Database, engine: &mut Engine) -> Result<EngineState, ErrorType> {
        let tasks1 = db.claim_runnable_tasks::<Worker1Job>(None)?;
        let tasks1_count =  tasks1.len();

        if tasks1_count > 0 {
            info!("Found {} tasks for worker1", tasks1_count);
        }
        for task in tasks1 {
            engine.worker1.send_task(task)?;
        }

        let tasks2 = db.claim_runnable_tasks::<Worker2Job>(None)?;
        let tasks2_count = tasks2.len();

        if tasks2_count > 0 {
            info!("Found {} tasks for worker2", tasks2_count);
        }
        for task in tasks2 {
            engine.worker2.send_task(task)?;
        }

        Ok(if tasks1_count == 0 && tasks2_count == 0 {
            EngineState::Idle
        } else {
            EngineState::WorkDone
        })
    }

    fn find_failed_tasks(db: &mut Database) -> Result<EngineState, ErrorType> {
        // todo check if there are any timeouted tasks.

        Ok(EngineState::Idle)
    }

    fn body(db: &mut Database, engine: &mut Engine) -> Result<(), ErrorType> {
        loop {
            let claiming_result = claim_tasks(db, engine)?;
            
            let finding_failed_result = find_failed_tasks(db)?;

            check_if_workers_are_workin(engine);

            match (claiming_result, finding_failed_result) {
                (EngineState::Idle, EngineState::Idle) => {
                    thread::sleep(Duration::from_secs(1));
                    continue;
                },
                (_, _) => {
                    continue;
                },
            }
        
        }
    }

    let mut engine = Engine::new();
    let mut db =  try_open_connection();
    
    engine.start_failed_workers();

    loop {        
        match body(&mut db, &mut engine) {
            Ok(()) => continue,
            Err(e) => {
                error!("{}", e);

                match e {
                    ErrorType::DatabaseConnectionError(_) => {
                        warn!("Reseting connection with database...");
                        db = try_open_connection();
                        warn!("Reseting worker threads...");
                        engine.start_failed_workers();
                    },
                    ErrorType::WorkerThreadFailed => {
                        warn!("Reseting worker threads...");
                        engine.start_failed_workers();
                    },
                    _ => continue,
                }
            }
        }
    }
}

pub fn run() -> std::thread::JoinHandle<()> {
    std::thread::spawn(|| schedule_thread_body())
}

impl Engine {
    pub fn new() -> Self {
        Self {
            worker1: WorkerThread::new(),
            worker2: WorkerThread::new(),
        }
    }

    pub fn start_failed_workers(&mut self){
        self.worker1.restore_thread(|| (Worker1::new(), try_open_connection()));
        self.worker2.restore_thread(|| (Worker2::new(), try_open_connection()));
    }
}