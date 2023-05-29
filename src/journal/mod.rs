use crate::{database::common, processing::{task::Task, job::WorkerJob}};



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

   
}