use std::time::SystemTime;

use crate::{
    database::{
        common::{Database, ErrorType},
        schema,
    },
    processing::job::JobType,
};

#[derive(Debug)]
pub struct Task {
    pub id: i64,
    pub task_id: i64,
    pub parent_tasks: Option<Vec<Task>>,
    pub status: schema::Status,
    pub timestamp: i64,
    pub data: String,
    pub params: JobType,
}

pub struct InsertableTaskTree {
    pub parent_tasks: Vec<InsertableTaskTree>,
    pub status: schema::Status,
    pub data: Option<String>,
    pub params: JobType,
}

fn get_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

mod task_querry {
    use super::Task;
    use crate::{
        database::{common::ErrorType, schema, repositories::task::get_timestamp},
        processing::job::JobType,
    };
    use postgres::{GenericClient};

    pub fn get_task_by_id<C>(conn: &mut C, id: i64) -> Result<schema::TaskSchema, ErrorType> 
    where C: GenericClient 
    {
        const QUERY: &str =
            "SELECT id, task_id, status, timestamp, data, params FROM tasks WHERE id = $1";

        let row = conn.query_one(QUERY, &[&id])?;

        Ok(schema::TaskSchema {
            id: row.try_get(0)?,
            task_id: row.try_get(1)?,
            status: row.try_get(2)?,
            timestamp: row.try_get(3)?,
            data: row.try_get(4)?,
            params: row.try_get(5)?,
        })
    }

    pub fn get_runnable_tasks(conn: &mut impl GenericClient) -> Result<Vec<Task>, ErrorType> {
        const QUERRY: &str = "SELECT t.id, t.task_id, status, timestamp, data, params FROM tasks t LEFT JOIN parents p ON t.task_id = p.task_id WHERE (t.status = 'pending' AND (p.parent_id IS NULL OR p.parent_id IN (SELECT task_id FROM tasks WHERE status = 'completed')))";

        let rows = conn.query(QUERRY, &[])?;

        let mut tasks = Vec::new();

        for row in rows {
            let id: i64 = row.try_get(0)?;
            let task_id: i64 = row.try_get(1)?;
            let status: schema::Status = row.try_get(2)?;
            let timestamp: i64 = row.try_get(3)?;
            let data: String = row.try_get(4)?;
            let params: String = row.try_get(5)?;

            let params: JobType = serde_json::from_str(&params)?;

            tasks.push(Task {
                id,
                task_id,
                parent_tasks: None,
                status,
                timestamp,
                data,
                params,
            });
        }

        Ok(tasks)
    }

    pub fn get_last_task_state(conn: &mut impl GenericClient, task_id: i64) -> Result<Task, ErrorType> {
        const QUERY: &str = "SELECT id, task_id, status, timestamp, data, params FROM tasks WHERE task_id = $1 ORDER BY timestamp DESC LIMIT 1";
        
        let row = conn.query_one(QUERY, &[&task_id])?;

        let id: i64 = row.try_get(0)?;
        let task_id: i64 = row.try_get(1)?;
        let status: schema::Status = row.try_get(2)?;
        let timestamp: i64 = row.try_get(3)?;
        let data: String = row.try_get(4)?;
        let params: String = row.try_get(5)?;
        let params: JobType = serde_json::from_str(&params)?;
        
        Ok(Task {
            id,
            task_id,
            parent_tasks: None,
            status,
            timestamp,
            data,
            params,
        })
    }

    pub fn is_task_not_completed(conn: &mut impl GenericClient, task_id: i64) -> Result<bool, ErrorType> {
        use schema::Status;
        
        let task = get_last_task_state(conn, task_id)?;

        match task.status {
            Status::Pending|Status::Failed => Ok(true),
            _ => Ok(false),
        }
    }

    pub fn insert_status(conn: &mut impl GenericClient, task_id: i64, status: schema::Status) -> Result<(), ErrorType> {
        const QUERY: &str = "INSERT INTO tasks (task_id, status, timestamp) VALUES ($1, $2, $3)";

        let timestamp = get_timestamp();

        conn.execute(QUERY, &[&task_id, &status, &timestamp])?;

        Ok(())
    }

    pub fn mark_task_as_running(conn: &mut impl GenericClient, task: &Task) -> Result<(), ErrorType> {
        // check if task is runnable
        if is_task_not_completed(conn, task.task_id)? {
            insert_status(conn, task.task_id, schema::Status::Running)?;
    
            Ok(())
        } else {
            Err(ErrorType::TaskNotRunnable(task.task_id))
        }
    }
}

impl Database {
    pub fn insert_new_task_tree(&mut self, task: &InsertableTaskTree) -> Result<(), ErrorType> {
        const QUERY: &str = "INSERT INTO tasks (task_id, status, timestamp, data, params) VALUES ($1, $2, $3, $4, $5)";
        const QUERY2: &str = "INSERT INTO parents (task_id, parent_id) VALUES ($1, $2)";

        fn insert_task(
            tx: &mut postgres::Transaction,
            task: &InsertableTaskTree,
            timestamp: i64,
        ) -> Result<i64, ErrorType> {
            // get next free task_id
            let task_id = tx.query_one("SELECT nextval('task_id_seq')", &[])?;
            let task_id: i64 = task_id.try_get(0)?;

            // insert task
            tx.execute(
                QUERY,
                &[
                    &task_id,
                    &task.status,
                    &timestamp,
                    &task.data,
                    &serde_json::to_string(&task.params)?,
                ],
            )?;

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

        let timestamp = get_timestamp();

        let mut tx = self.conn.transaction()?;
        insert_task(&mut tx, task, timestamp)?;
        tx.commit()?;

        Ok(())
    }

    pub fn get_task_by_id(&mut self, id: i64) -> Result<schema::TaskSchema, ErrorType> {
        task_querry::get_task_by_id(&mut self.conn, id)
    }

    pub fn get_runnable_tasks(&mut self) -> Result<Vec<Task>, ErrorType> {
        task_querry::get_runnable_tasks(&mut self.conn)
    }

    pub fn get_last_task_state(&mut self, task_id: i64) -> Result<Task, ErrorType> {
        task_querry::get_last_task_state(&mut self.conn, task_id)
    }



    pub fn claim_runnable_tasks<WorkerJobType: TryFrom<JobType> + Copy>(
        &mut self,
        limit: Option<u32>,
    ) -> Result<Vec<Task>, ErrorType> {
        let mut tx = self.transaction()?;

        let tasks = task_querry::get_runnable_tasks(&mut tx)?;

        let tasks = tasks
            .iter()
            .filter_map(|task| {
                task.params.try_into().ok().map(|_: WorkerJobType| Task {
                    id: task.id,
                    task_id: task.task_id,
                    parent_tasks: None,
                    status: task.status,
                    timestamp: task.timestamp,
                    data: task.data.clone(),
                    params: task.params,
                })
            })
            .take(limit.unwrap_or(std::u32::MAX) as usize)
            .collect::<Vec<_>>();

        // claim tasks
        for task in &tasks {
            task_querry::mark_task_as_running(&mut tx, task.clone())?;
        }

        tx.commit()?;

        Ok(tasks)
    }

    pub fn claim_all_runnable_tasks<WorkerJobType: TryFrom<JobType> + Copy>(
        &mut self,
    ) -> Result<Vec<Task>, ErrorType> {
        self.claim_runnable_tasks::<WorkerJobType>(None)
    }
}   
