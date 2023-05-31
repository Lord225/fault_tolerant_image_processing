pub mod common;
pub mod migration;
pub mod schema;

pub mod repositories {
    pub mod task;
}

#[cfg(test)]
mod tests {
    use clap::__derive_refs::once_cell::sync::Lazy;

    use crate::processing::job::JobType;
    use crate::processing::worker::worker1::Worker1Job;
    use crate::processing::worker::worker2::Worker2Job;

    use super::common;
    use super::migration;
    use super::repositories::task::InsertableTaskTree;
    use super::schema::Status;

    use serial_test::serial;

    static EXAMPLE_TASK_TREE1: Lazy<InsertableTaskTree> = Lazy::new(|| InsertableTaskTree {
        data: Some("Main Task".to_string()),
        status: Status::Pending,
        params: JobType::new_blur(0.0),

        parent_tasks: vec![
            InsertableTaskTree {
                data: Some("Subtask 1".to_string()),
                status: Status::Pending,
                params: JobType::new_blur(0.0),
                parent_tasks: vec![InsertableTaskTree {
                    data: Some("Subtask for subtask 1".to_string()),
                    status: Status::Completed,
                    params: JobType::new_blur(0.0),

                    parent_tasks: vec![],
                }],
            },
            InsertableTaskTree {
                data: Some("Subtask 2".to_string()),
                status: Status::Pending,
                params: JobType::new_resize(100, 100),

                parent_tasks: vec![],
            },
        ],
    });

    fn init_database() -> common::Database {
        common::reset_database().unwrap();
        common::open_connection()
            .map(|mut db| {
                migration::run_migrations(&mut db);
                db
            })
            .unwrap()
    }

    #[test]
    #[serial]
    fn test_runnable_tasks() {
        let mut db = init_database();

        db.insert_new_task_tree(&EXAMPLE_TASK_TREE1).unwrap();

        let runnable = db.get_runnable_tasks().unwrap();

        assert_eq!(runnable.len(), 2);

        assert!(runnable.iter().any(|x| x.data == *"Subtask 1"));
        assert!(runnable.iter().any(|x| x.data == *"Subtask 2"));   
    }

    #[test]
    #[serial]
    fn claim_one_runnable_task() {
        let mut db = init_database();

        db.insert_new_task_tree(&EXAMPLE_TASK_TREE1).unwrap();

        let tasks_for_worker_1 = db.claim_runnable_tasks::<Worker1Job>(Some(1));
        let tasks_for_worker_2 = db.claim_runnable_tasks::<Worker2Job>(Some(1));

        assert!(tasks_for_worker_1.unwrap().first().unwrap().data == *"Subtask 2");
        assert!(tasks_for_worker_2.unwrap().first().unwrap().data == *"Subtask 1");
    }
}
