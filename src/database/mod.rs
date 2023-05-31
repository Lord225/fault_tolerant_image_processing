pub mod common;
pub mod migration;
pub mod schema;

pub mod repositories {
    pub mod task;
}

#[cfg(test)]
mod tests {
    use crate::processing::worker::worker1::Worker1Job;
    use crate::processing::worker::worker2::Worker2Job;
    use crate::tests_common::*;

    use serial_test::serial;

    #[test]
    #[serial]
    fn test_runnable_tasks() {
        let mut db = init_database();

        db.insert_new_task_tree(&EXAMPLE_TASK_TREE1).unwrap();

        let runnable = db.get_runnable_tasks().unwrap();

        assert_eq!(runnable.len(), 2);

        assert!(runnable.iter().any(|x| x.data == Some("Subtask 1".to_string())));
        assert!(runnable.iter().any(|x| x.data == Some("Subtask 2".to_string())));   
    }

    #[test]
    #[serial]
    fn claim_one_runnable_task() {
        let mut db = init_database();

        db.insert_new_task_tree(&EXAMPLE_TASK_TREE1).unwrap();

        let tasks_for_worker_1 = db.claim_runnable_tasks::<Worker1Job>(Some(1));
        let tasks_for_worker_2 = db.claim_runnable_tasks::<Worker2Job>(Some(1));

        assert!(tasks_for_worker_1.unwrap().first().unwrap().data == Some("Subtask 1".to_string()));
        assert!(tasks_for_worker_2.unwrap().first().unwrap().data == Some("Subtask 1".to_string()));
    }
}
