use std::time::SystemTime;

use crate::{
    database::{
        common::{Database, ErrorType},
        schema,
    },
    processing::{job::WorkerJob, task::Task},
};

#[derive(Debug)]
pub struct TaskTree {
    id: i64,
    task_id: i64,
    parent_tasks: Option<Vec<TaskTree>>,
    status: schema::Status,
    timestamp: i64,
    data: String,
    params: WorkerJob,
}

pub struct InsertableTaskTree {
    pub parent_tasks: Vec<InsertableTaskTree>,
    pub status: schema::Status,
    pub data: Option<String>,
    pub params: WorkerJob,
}

fn get_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

mod task_querry {
    use super::TaskTree;
    use crate::{
        database::{common::ErrorType, schema, repositories::task::get_timestamp},
        processing::job::WorkerJob,
    };
    use postgres::Transaction;

    pub fn get_runnable_tasks(tx: &mut Transaction) -> Result<Vec<TaskTree>, ErrorType> {
        const QUERRY: &str = "SELECT t.id, t.task_id, status, timestamp, data, params FROM tasks t LEFT JOIN parents p ON t.task_id = p.task_id WHERE (t.status = 'pending' AND (p.parent_id IS NULL OR p.parent_id IN (SELECT task_id FROM tasks WHERE status = 'completed')))";

        let rows = tx.query(QUERRY, &[])?;

        let mut tasks = Vec::new();

        for row in rows {
            let id: i64 = row.try_get(0)?;
            let task_id: i64 = row.try_get(1)?;
            let status: schema::Status = row.try_get(2)?;
            let timestamp: i64 = row.try_get(3)?;
            let data: String = row.try_get(4)?;
            let params: String = row.try_get(5)?;

            let params: WorkerJob = serde_json::from_str(&params)?;

            tasks.push(TaskTree {
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

    pub fn mark_task_as_running(tx: &mut Transaction, task: &TaskTree) -> Result<(), ErrorType> {
        const QUERY: &str = "INSERT INTO tasks (task_id, status, timestamp, data, params) VALUES ($1, $2, $3, $4, $5)";

        let timestamp = get_timestamp();

        // insert task
        tx.execute(
            QUERY,
            &[
                &task.task_id,
                &schema::Status::Running,
                &timestamp,
                &task.data,
                &serde_json::to_string(&task.params)?,
            ],
        )?;


        Ok(())
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
        const QUERY: &str =
            "SELECT id, task_id, status, timestamp, data, params FROM tasks WHERE id = $1";

        let row = self.query_one(QUERY, &[&id])?;

        Ok(schema::TaskSchema {
            id: row.try_get(0)?,
            task_id: row.try_get(1)?,
            status: row.try_get(2)?,
            timestamp: row.try_get(3)?,
            data: row.try_get(4)?,
            params: row.try_get(5)?,
        })
    }

    pub fn get_runnable_tasks(&mut self) -> Result<Vec<TaskTree>, ErrorType> {
        let mut tx = self.conn.transaction()?;
        let tasks = task_querry::get_runnable_tasks(&mut tx)?;
        tx.commit()?;

        Ok(tasks)
    }

    pub fn is_task_pending(&mut self, task_id: i64) -> Result<bool, ErrorType> {
        const QUERY: &str =
            "SELECT status FROM tasks WHERE task_id = $1 ORDER BY timestamp DESC LIMIT 1";

        let row = self.query_one(QUERY, &[&task_id])?;

        Ok(row.try_get::<_, schema::Status>(0)? == schema::Status::Pending)
    }

    pub fn claim_runnable_tasks<JobType: TryFrom<WorkerJob> + Copy>(
        &mut self,
        limit: Option<u32>,
    ) -> Result<Vec<TaskTree>, ErrorType> {
        let mut tx = self.transaction()?;

        let tasks = task_querry::get_runnable_tasks(&mut tx)?;

        let tasks = tasks
            .iter()
            .filter_map(|task| {
                task.params.try_into().ok().map(|_: JobType| TaskTree {
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


        Ok(tasks)
    }

    pub fn claim_all_runnable_tasks<JobType: TryFrom<WorkerJob> + Copy>(
        &mut self,
    ) -> Result<Vec<TaskTree>, ErrorType> {
        self.claim_runnable_tasks::<JobType>(None)
    }
}   
