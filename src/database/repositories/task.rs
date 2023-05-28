use std::{time::SystemTime};

use crate::{workers::task::Job, 
            database::{schema, 
                       common::{Database, ErrorType}}};

#[derive(Debug)]
pub struct Task {
    id: i64,
    parent_tasks: Option<Vec<Task>>,
    status: schema::Status,
    timestamp: i64,
    data: String,
    params: Job,
}

pub struct InsertableTask {
    pub parent_tasks: Vec<InsertableTask>,
    pub status: schema::Status,
    pub data: Option<String>,
    pub params: Job,
}

impl Database {
    pub fn insert_new_task_tree(&mut self, task: &InsertableTask) -> Result<(), ErrorType> 
    {
        const QUERY: &str = "INSERT INTO tasks (task_id, status, timestamp, data, params) VALUES ($1, $2, $3, $4, $5)";
        const QUERY2: &str = "INSERT INTO parents (task_id, parent_id) VALUES ($1, $2)";

        fn insert_task(tx: &mut postgres::Transaction, task: &InsertableTask, timestamp: i64) -> Result<i64, ErrorType> {
            // get next free task_id
            let task_id = tx.query_one("SELECT nextval('task_id_seq')", &[])?;
            let task_id: i64 = task_id.try_get(0)?;
            
            // insert task
            tx.execute(QUERY, &[&task_id, &task.status, &timestamp, &task.data, &serde_json::to_string(&task.params)?])?;

            let mut parent_ids = Vec::new();

            // insert parents
            for parent in &task.parent_tasks {
                parent_ids.push(insert_task(tx, parent, timestamp)?);
            }

            // insert parent relations
            for parent_id in parent_ids {
                tx.execute(QUERY2, &[&task_id, &parent_id])?;
            }

            Ok(task_id)
        }

        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64;
        
        let mut tx = self.conn.transaction()?;
        insert_task(&mut tx, task, timestamp)?;
        tx.commit()?;

        Ok(())
       
    }

    pub fn get_task_by_id(&mut self, id: i64) -> Result<schema::TaskSchema, ErrorType> {
        const QUERY: &str = "SELECT id, task_id, status, timestamp, data, params FROM tasks WHERE id = $1";

        let row = self.query_one(QUERY, &[&id])?;

        Ok(
            schema::TaskSchema {
                id: row.try_get(0)?,
                task_id: row.try_get(1)?,
                status: row.try_get(2)?,
                timestamp: row.try_get(3)?,
                data: row.try_get(4)?,
                params: row.try_get(5)?,
            }
        )
    }

    pub fn get_runnable_tasks(&mut self) -> Result<Vec<Task>, ErrorType> {
        const QUERRY: &str = "SELECT * FROM tasks t LEFT JOIN parents p ON t.task_id = p.task_id WHERE (t.status = 'pending' AND (p.parent_id IS NULL OR p.parent_id IN (SELECT task_id FROM tasks WHERE status = 'completed')))";

        let rows = self.query(QUERRY, &[])?;
        
        for row in rows {
            println!("{:?}", row);
        }

        Ok(Vec::new())
    }
    
    pub fn is_task_pending(&mut self, task_id: i64) -> Result<bool, ErrorType> {
        const QUERY: &str = "SELECT status FROM tasks WHERE task_id = $1 ORDER BY timestamp DESC LIMIT 1";

        let row = self.query_one(QUERY, &[&task_id])?;

        Ok(
            row.try_get::<_, schema::Status>(0)? == schema::Status::Pending
        )
    }

    
}