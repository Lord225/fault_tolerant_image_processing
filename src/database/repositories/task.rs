use std::time::SystemTime;

use crate::{
    database::{
        common::{Database, ErrorType},
        schema,
    },
    processing::job::JobType,
};

/// Represents a task or an Task tree.
/// Task is a single unit of work marked with status, timestamp and data and params.
#[derive(Debug)]
pub struct Task {
    /// Unique Id for task and state of the task.
    id: i64,
    /// Unique Id of task
    pub task_id: i64,
    /// Parent tasks of this task. If None then it is undefined if task has parents or not.
    pub parent_tasks: Option<Vec<Task>>,
    /// Status of the task.
    pub status: schema::Status,
    /// Timestamp - unix
    pub timestamp: i64,
    /// output of given task. None if task hasnt been completed yet.
    pub data: Option<String>,
    /// params of given task.
    pub params: JobType,
}

pub struct InsertableTaskTree {
    pub parent_tasks: Vec<InsertableTaskTree>,
    pub status: schema::Status,
    pub data: Option<String>,
    pub params: JobType,
}

impl InsertableTaskTree {
    pub fn input(data: &str) -> Self {
        Self {
            parent_tasks: vec![],
            status: schema::Status::Pending,
            data: Some(data.to_string()),
            params: JobType::input(),
        }
    }
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

    pub fn get_parent_tasks(conn: &mut impl GenericClient, child_task_id: i64) -> Result<Vec<Task>, ErrorType> {
        // get all parent tasks 
        // it means: select all tasks that have parents with child_task_id is in parents table (sub select)
        const QUERY: &str = "SELECT t.id, t.task_id, status, timestamp, data, params FROM tasks t WHERE t.task_id IN (SELECT parent_id FROM parents WHERE task_id = $1)";

        let rows = conn.query(QUERY, &[&child_task_id])?;

        let mut tasks = Vec::new();

        for row in rows {
            let id: i64 = row.try_get(0)?;
            let task_id: i64 = row.try_get(1)?;
            let status: schema::Status = row.try_get(2)?;
            let timestamp: i64 = row.try_get(3)?;
            let data: Option<String> = row.try_get(4)?;
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

    pub fn get_runnable_tasks(conn: &mut impl GenericClient) -> Result<Vec<Task>, ErrorType> {
        // select tasks that have no parents, or ALL parents are completed
        const QUERRY: &str = r#"SELECT t.id, t.task_id, t.status, t.timestamp, t.data, t.params
        FROM tasks t
        WHERE t.status = 'pending';"#; // bad

        let rows = conn.query(QUERRY, &[])?;

        let mut tasks = Vec::new();

        for row in rows {
            let id: i64 = row.try_get(0)?;
            let task_id: i64 = row.try_get(1)?;
            let status: schema::Status = row.try_get(2)?;
            let timestamp: i64 = row.try_get(3)?;
            let data: Option<String> = row.try_get(4)?;
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
        let data: Option<String> = row.try_get(4)?;
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

    pub fn insert_status(conn: &mut impl GenericClient, task: &Task, status: schema::Status) -> Result<(), ErrorType> {
        const QUERY: &str = "INSERT INTO tasks (task_id, status, timestamp, data, params) VALUES ($1, $2, $3, $4, $5)";

        let timestamp = get_timestamp();

        conn.execute(QUERY, &[&task.task_id, &status, &timestamp, &task.data, &serde_json::to_string(&task.params)?])?;

        Ok(())
    }

    pub fn mark_task_as_running(conn: &mut impl GenericClient, task: &Task) -> Result<(), ErrorType> {
        // check if task is runnable
        if is_task_not_completed(conn, task.task_id)? {
            insert_status(conn, task, schema::Status::Running)?;
    
            Ok(())
        } else {
            Err(ErrorType::TaskNotRunnable(task.task_id))
        }
    }

    pub fn mark_task_as_failed(conn: &mut impl GenericClient, task: &Task) -> Result<(), ErrorType> {
        todo!()
    }

    pub fn mark_task_as_completed(conn: &mut impl GenericClient, task: &Task) -> Result<(), ErrorType> {
        todo!()
    }

    pub fn search_for_timeouted(conn: &mut impl GenericClient, timeout: std::time::Duration) -> Result<Vec<Task>, ErrorType> {
        todo!()
    }

    pub fn mark_timeouted_tasks_as_failed(conn: &mut impl GenericClient, tasks: Vec<Task>) -> Result<(), ErrorType> {
        todo!()
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

    pub fn get_runnable_tasks(&mut self) -> Result<Vec<Task>, ErrorType> {
        task_querry::get_runnable_tasks(&mut self.conn)
    }

    pub fn get_last_task_state(&mut self, task_id: i64) -> Result<Task, ErrorType> {
        task_querry::get_last_task_state(&mut self.conn, task_id)
    }

    pub fn get_parent_tasks(&mut self, task_id: i64) -> Result<Vec<Task>, ErrorType> {
        task_querry::get_parent_tasks(&mut self.conn, task_id)
    }

    pub fn mark_task_as_completed(&mut self, task_id: i64) -> Result<(), ErrorType> {
        let mut tx = self.conn.transaction()?;

        let task = task_querry::get_last_task_state(&mut tx, task_id)?;

        if task.status != schema::Status::Running {
            return Err(ErrorType::Other);
        }

        task_querry::insert_status(&mut tx, &task, schema::Status::Completed)?;

        tx.commit()?;

        Ok(())
    }

    pub fn mark_task_as_failed(&mut self, task_id: i64) -> Result<(), ErrorType> {
        let mut tx = self.conn.transaction()?;

        let task = task_querry::get_last_task_state(&mut tx, task_id)?;

        task_querry::insert_status(&mut tx, &task, schema::Status::Failed)?;

        tx.commit()?;

        Ok(())
    }

    pub fn claim_runnable_tasks<WorkerJobType: TryFrom<JobType> + Copy>(
        &mut self,
        limit: Option<u32>,
    ) -> Result<Vec<Task>, ErrorType> {
        let mut tx = self.transaction()?;

        let tasks = task_querry::get_runnable_tasks(&mut tx)?;

        dbg!(&tasks);

        let tasks = tasks
            .iter()
            .filter_map(|task| {
                task.params
                .try_into()
                .ok()
                .map(|_: WorkerJobType| Task {
                    id: task.id,
                    task_id: task.task_id,
                    parent_tasks: Some(task_querry::get_parent_tasks(&mut tx, task.task_id).unwrap()),
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
            task_querry::mark_task_as_running(&mut tx, task)?;
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
