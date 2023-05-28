use postgres_derive::{FromSql, ToSql};

#[derive(Debug, PartialEq, Eq, Clone, Copy, FromSql, ToSql)]
#[postgres(name = "status_type")]
pub enum Status {
    #[postgres(name = "pending")]
    Pending,
    #[postgres(name = "running")]
    Running,
    #[postgres(name = "completed")]
    Completed,
    #[postgres(name = "failed")]
    Failed,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TaskSchema {
    pub id: i64,
    pub task_id: i64,
    pub status: Status,
    pub timestamp: i64,
    pub data: String,
    pub params: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParentSchema {
    pub task_id: i64,
    pub parent_id: i64,
}
