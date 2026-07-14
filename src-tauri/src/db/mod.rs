use rusqlite::{Connection, Result as SqliteResult};
use thiserror::Error;

pub mod migrations;
pub mod scan_config;

const SCHEMA_VERSION: i32 = 3;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("database lock poisoned")]
    LockPoisoned,
}

pub struct Database {
    conn: std::sync::Mutex<Connection>,
}

impl Database {
    pub fn open(db_path: &std::path::Path) -> Result<Self, DatabaseError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;",
        )?;
        migrations::initialize_schema(&conn)?;

        Ok(Self {
            conn: std::sync::Mutex::new(conn),
        })
    }

    pub fn with_connection<T, F>(&self, f: F) -> Result<T, DatabaseError>
    where
        F: FnOnce(&Connection) -> SqliteResult<T>,
    {
        let guard = self.conn.lock().map_err(|_| DatabaseError::LockPoisoned)?;
        f(&guard).map_err(DatabaseError::from)
    }

    pub fn execute(&self, sql: &str, params: impl rusqlite::Params) -> Result<usize, DatabaseError> {
        self.with_connection(|conn| conn.execute(sql, params))
    }

    pub fn schema_version(&self) -> Result<i32, DatabaseError> {
        self.with_connection(|conn| {
            conn.query_row("PRAGMA user_version", [], |row| row.get(0))
        })
    }
}

pub fn expected_schema_version() -> i32 {
    SCHEMA_VERSION
}
