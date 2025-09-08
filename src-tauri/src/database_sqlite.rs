use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result as SqliteResult, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub file_path: String,
    pub file_type: String,
    pub file_size: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub user_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub details: String,
    pub created_at: DateTime<Utc>,
}

pub struct Database {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl Database {
    pub fn new(db_path: PathBuf) -> SqliteResult<Self> {
        let conn = Connection::open(&db_path)?;
        
        conn.execute_batch(r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = 10000;
            PRAGMA busy_timeout = 30000;
            PRAGMA wal_autocheckpoint = 1000;
            PRAGMA mmap_size = 268435456;
            PRAGMA temp_store = memory;
        "#)?;
        
        let database = Database {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        };
        
        database.create_tables()?;
        Ok(database)
    }
    
    fn create_tables(&self) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_login TEXT
            )
        "#, [])?;
        
        conn.execute(r#"
            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                name TEXT NOT NULL,
                file_path TEXT NOT NULL,
                file_type TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                tags TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users (id)
            )
        "#, [])?;
        
        conn.execute(r#"
            CREATE TABLE IF NOT EXISTS activities (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                action TEXT NOT NULL,
                resource_type TEXT NOT NULL,
                resource_id TEXT NOT NULL,
                details TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users (id)
            )
        "#, [])?;
        
        conn.execute("CREATE INDEX IF NOT EXISTS idx_documents_user_id ON documents(user_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_documents_created_at ON documents(created_at)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_activities_user_id ON activities(user_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_activities_created_at ON activities(created_at)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)", [])?;
        
        Ok(())
    }
    
    fn execute_with_retry<F, R>(&self, operation: F) -> SqliteResult<R> 
    where
        F: Fn(&Connection) -> SqliteResult<R>,
    {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY_MS: u64 = 100;
        
        let mut last_error = None;
        
        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                thread::sleep(Duration::from_millis(RETRY_DELAY_MS * attempt as u64));
            }
            
            match self.conn.lock() {
                Ok(conn) => {
                    match operation(&*conn) {
                        Ok(result) => return Ok(result),
                        Err(e) => {
                            last_error = Some(e);
                            if let Some(rusqlite::Error::SqliteFailure(err, _)) = last_error.as_ref() {
                                if err.code == rusqlite::ErrorCode::DatabaseBusy || 
                                   err.code == rusqlite::ErrorCode::DatabaseLocked {
                                    continue;
                                }
                            }
                            return Err(last_error.unwrap());
                        }
                    }
                },
                Err(_) => continue,
            }
        }
        
        Err(last_error.unwrap_or_else(|| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ErrorCode::DatabaseBusy as i32),
                Some("Max retries exceeded".to_string())
            )
        }))
    }

    // ... [restante das funções create_user, get_user_by_username, etc. continuam iguais]
}
