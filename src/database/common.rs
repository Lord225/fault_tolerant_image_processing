use std::{ops::{Deref, DerefMut}, fmt::Display};

use postgres::{Client, NoTls, Error};


pub struct Database {
    pub conn: Client,
}


pub fn open_connection() -> Result<Database, Error> {
    let conn = Client::connect(&std::env::var("DATABASE_URL").unwrap_or("postgres://postgres:root@localhost:5432/images".into()), NoTls)?;
    
    Ok(Database { conn  })
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
    NotFound,
    SerializationError,
    Other,
}

// pozwala na konwersje błędów z postgres na własne błędy
impl From<postgres::Error> for ErrorType {
    fn from(e: postgres::Error) -> Self {
        match e.code() {
            Some(code) => match code.code() {
                "23505" => ErrorType::DatabaseError(e.to_string()),
                "23503" => ErrorType::NotFound,
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
            ErrorType::DatabaseError(message) => write!(f, "Database Error: {}", message),
            ErrorType::NotFound => write!(f, "Entry Not Found"),
            ErrorType::Other => write!(f, "Other Logic Error"),
            ErrorType::SerializationError => write!(f, "Serialization Error"),
        }
    }
}

impl std::error::Error for ErrorType {}

impl From<serde_json::Error> for ErrorType {
    fn from(_: serde_json::Error) -> Self {
        ErrorType::SerializationError
    }
}
