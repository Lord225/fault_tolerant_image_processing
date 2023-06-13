use log::{info, error, warn};

use std::sync::Arc;
use std::{thread, sync::RwLock};
use std::time::Duration;
use crate::{processing::worker::{worker1::{Worker1, Worker1Job}, worker2::{Worker2, Worker2Job}, WorkerThread, WorkerErrorConfig}, database::common::{try_open_connection, Database, ErrorType}};
const TIMEOUT_DURATION: std::time::Duration = Duration::from_secs(2);

pub type ConfigType = Arc<RwLock<WorkerErrorConfig>>;
struct Engine {
    worker1: WorkerThread<Worker1>,
    worker2: WorkerThread<Worker2>,
    config: ConfigType,
}

enum EngineState {
    WorkDone,
    Idle,
}

fn schedule_thread_body(config: ConfigType) -> !
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
        
        let failed_count = db.mark_as_failed_timeouted(TIMEOUT_DURATION)?;

        if failed_count > 0 {
            warn!("Found {} failed tasks", failed_count)
        }

        Ok(if failed_count == 0 {
            EngineState::Idle
        } else {
            EngineState::WorkDone
        })
    }

    fn body(db: &mut Database, engine: &mut Engine, config: &ConfigType) -> Result<(), ErrorType> {
        loop {
            let claiming_result = claim_tasks(db, engine)?;
            
            let finding_failed_result = find_failed_tasks(db)?;

            check_if_workers_are_workin(engine);

            match (claiming_result, finding_failed_result) {
                (EngineState::Idle, EngineState::Idle) => {
                    thread::sleep(Duration::from_millis(250));
                    continue;
                },
                (_, _) => {
                    thread::sleep(Duration::from_millis(config.read().unwrap().throttle.as_millis() as u64));
                    continue;
                },
            }
        
        }
    }

    let mut engine = Engine::new(config.clone());
    let mut db =  try_open_connection();
    
    engine.start_failed_workers();

    loop {        
        match body(&mut db, &mut engine, &config) {
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

pub fn run() -> (std::thread::JoinHandle<()>, ConfigType) {
    let config = Arc::new(RwLock::new(WorkerErrorConfig::default()));
    let config_clone = config.clone();
    (std::thread::spawn(|| schedule_thread_body(config)), config_clone)
}

impl Engine {
    pub fn new(config: ConfigType) -> Self {
        Self {
            worker1: WorkerThread::new(),
            worker2: WorkerThread::new(),
            config,
        }
    }

    pub fn start_failed_workers(&mut self){
        self.worker1.restore_thread(|| (Worker1::new(), try_open_connection(), self.config.clone()));
        self.worker2.restore_thread(|| (Worker2::new(), try_open_connection(), self.config.clone()));
    }
}