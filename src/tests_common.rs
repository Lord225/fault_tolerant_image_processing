use clap::__derive_refs::once_cell::sync::Lazy;

use crate::database::common;
use crate::database::migration;
use crate::database::repositories::task::InsertableTaskTree;
use crate::database::schema::Status;
use crate::processing::job::JobType;

#[allow(unused)]
pub static EXAMPLE_TASK_TREE1: Lazy<InsertableTaskTree> = Lazy::new(|| InsertableTaskTree {
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

#[allow(unused)]
pub fn init_database() -> common::Database {
    common::reset_database().unwrap();
    common::open_connection()
        .map(|mut db| {
            migration::run_migrations(&mut db);
            db
        })
        .unwrap()
}