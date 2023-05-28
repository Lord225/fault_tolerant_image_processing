use crate::{database::common, workers::task::SubTask};



pub struct Journal {
    db: common::Database,
}



impl Journal {
    pub fn new() -> Self {
        let db = common::open_connection();
        
        Self {
            db: db.expect("Cannot open connection to database - This application requires postgresql to be running on port 5432"),
        }
    }

    ///
    /// Should find all tasks that are pending & possible to execute and return one
    pub fn search_pending_tasks(&mut self) -> Option<SubTask> {
        None
    }

    ///
    /// Should find all tasks that are failed and return one
    pub fn search_for_failed_tasks(&mut self) -> Vec<SubTask> {
        vec![]
    }


    ///
    /// Should mark given task as running
    pub fn mark_task_as_running(&mut self, task: &SubTask) {
        
    }

    ///
    /// Should mark given task as completed
    pub fn mark_task_as_completed(&mut self, task: &SubTask) {
        
    }

    ///
    /// Should mark given task as failed
    pub fn mark_task_as_failed(&mut self, task: &SubTask) {
        
    }


    ///
    /// Should find all tasks that are timeouted and mark them as failed
    pub fn mark_timeouted_tasks(&mut self) {
        
    }    
}