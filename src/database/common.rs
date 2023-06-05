use std::{ops::{Deref, DerefMut}, fmt::Display, time::Duration};

use log::{error, info, debug};
use postgres::{Client, NoTls, Error};


pub struct Database {
    pub conn: Client,
}

pub fn open_connection() -> Result<Database, Error> {
    let conn = Client::connect(&std::env::var("DATABASE_URL").unwrap_or("postgres://postgres:root@localhost:5432/images".into()), NoTls)?;
    
    debug!("Postgres Client opened successfully");

    Ok(Database { conn  })
}

pub fn try_open_connection() -> Database {
    loop {
        match open_connection() {
            Ok(conn) => return conn,
            Err(connectio_error) => error!("Unable to open SQL connection: {connectio_error}, retrying..."),
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

pub fn reset_database() -> Result<(), Error> {
    // connect to postgres
    let mut conn = Client::connect("postgres://postgres:root@localhost:5432/postgres", NoTls)?;

    // drop database
    conn.execute("DROP DATABASE IF EXISTS images", &[])?;

    // create database
    conn.execute("CREATE DATABASE images", &[])?;

    info!("Database reseted successfully");

    Ok(())
}

pub fn try_reset_database() {
    loop {
        match reset_database() {
            Ok(()) => return,
            Err(connectio_error) => error!("Unable to reset database: {connectio_error}, retrying..."),
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

impl Deref for Database {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl DerefMut for Database {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.conn
    }
}




#[derive(Debug, Clone)]
pub enum ErrorType {
    DatabaseError(String),
    DatabaseConnectionError(String),
    NotFound,
    WorkerThreadFailed,
    SerializationError,
    TaskNotRunnable(i64),
    Other,
}

// pozwala na konwersje błędów z postgres na własne błędy
impl From<postgres::Error> for ErrorType {
    fn from(e: postgres::Error) -> Self {
        match e.code() {
            Some(code) => match code.code() {
                c if c.starts_with("08") => ErrorType::DatabaseConnectionError(e.to_string()),
                _ => ErrorType::DatabaseError(e.to_string()),

            },
            None => ErrorType::DatabaseError(e.to_string()),
        }
    }
}
// pozwala na wyświetlanie błędów
impl Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorType::DatabaseError(message) => write!(f, "Database error: {}", message),
            ErrorType::DatabaseConnectionError(message) => write!(f, "Database connection error {}", message),
            ErrorType::NotFound => write!(f, "Entry Not Found"),
            ErrorType::Other => write!(f, "Other Logic Error"),
            ErrorType::SerializationError => write!(f, "Serialization Error"),
            ErrorType::TaskNotRunnable(task_id) => write!(f, "Task {} is not runnable", task_id),
            ErrorType::WorkerThreadFailed => write!(f, "Worker thread panicked!"),
        }
    }
}

impl std::error::Error for ErrorType {}

impl From<serde_json::Error> for ErrorType {
    fn from(_: serde_json::Error) -> Self {
        ErrorType::SerializationError
    }
}
