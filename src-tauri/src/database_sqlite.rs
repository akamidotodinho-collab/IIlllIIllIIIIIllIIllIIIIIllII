use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result as SqliteResult, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
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
        
        // CONFIGURAÇÕES CRÍTICAS DE PERFORMANCE E CONCORRÊNCIA
        conn.execute_batch(r#"
            -- WAL mode para melhor concorrência
            PRAGMA journal_mode = WAL;
            
            -- Sincronização otimizada
            PRAGMA synchronous = NORMAL;
            
            -- Cache aumentado (10MB)
            PRAGMA cache_size = 10000;
            
            -- Timeout para locks (30 segundos)
            PRAGMA busy_timeout = 30000;
            
            -- Auto checkpoint otimizado
            PRAGMA wal_autocheckpoint = 1000;
            
            -- Memory mapping (melhor I/O)
            PRAGMA mmap_size = 268435456;
            
            -- Temp store na memória
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
        
        // Tabela de usuários
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
        
        // Tabela de documentos
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
        
        // Tabela de atividades
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
        
        // ÍNDICES PARA PERFORMANCE
        conn.execute("CREATE INDEX IF NOT EXISTS idx_documents_user_id ON documents(user_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_documents_created_at ON documents(created_at)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_activities_user_id ON activities(user_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_activities_created_at ON activities(created_at)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)", [])?;
        
        Ok(())
    }
    
    // OPERAÇÃO COM RETRY AUTOMÁTICO - CORRIGIDO
    fn execute_with_retry<F, R>(&self, operation: F) -> SqliteResult<R> 
    where
        F: Fn(&Connection) -> SqliteResult<R>,
    {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY_MS: u64 = 100;
        
        let mut last_error = None;
        
        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                thread::sleep(Duration::from_millis(RETRY_DELAY_MS * (attempt as u64)));
            }
            
            match self.conn.lock() {
                Ok(conn) => {
                    match operation(&*conn) {
                        Ok(result) => return Ok(result),
                        Err(e) => {
                            last_error = Some(e);
                            // Se for erro de busy/lock, tenta novamente
                            if let Some(rusqlite::Error::SqliteFailure(err, _)) = last_error.as_ref() {
                                if err.code == rusqlite::ErrorCode::DatabaseBusy || 
                                   err.code == rusqlite::ErrorCode::DatabaseLocked {
                                    continue;
                                }
                            }
                            // Para outros erros, falha imediatamente
                            return Err(last_error.unwrap());
                        }
                    }
                },
                Err(_) => {
                    // Se não conseguir lock do Mutex, espera e tenta novamente
                    continue;
                }
            }
        }
        
        // CORREÇÃO CRÍTICA: Conversão correta para i32
        Err(last_error.unwrap_or_else(|| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ErrorCode::DatabaseBusy as i32),
                Some("Max retries exceeded".to_string())
            )
        }))
    }
    
    pub fn create_user(&self, user: &User) -> SqliteResult<()> {
        self.execute_with_retry(|conn| {
            conn.execute(
                "INSERT INTO users (id, username, email, password_hash, created_at, last_login) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    user.id,
                    user.username,
                    user.email,
                    user.password_hash,
                    user.created_at.to_rfc3339(),
                    user.last_login.map(|dt| dt.to_rfc3339())
                ]
            )?;
            Ok(())
        })
    }
    
    pub fn get_user_by_username(&self, username: &str) -> SqliteResult<Option<User>> {
        self.execute_with_retry(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, username, email, password_hash, created_at, last_login FROM users WHERE username = ?1"
            )?;
            
            let user_iter = stmt.query_map([username], |row| {
                let created_at_str: String = row.get(4)?;
                let last_login_str: Option<String> = row.get(5)?;
                
                Ok(User {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    email: row.get(2)?,
                    password_hash: row.get(3)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map_err(|_| rusqlite::Error::InvalidColumnType(4, "created_at".to_string(), rusqlite::types::Type::Text))?
                        .with_timezone(&Utc),
                    last_login: last_login_str.map(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now())
                    }),
                })
            })?;
            
            for user in user_iter {
                return Ok(Some(user?));
            }
            
            Ok(None)
        })
    }
    
    pub fn create_document(&self, document: &Document) -> SqliteResult<()> {
        self.execute_with_retry(|conn| {
            let tags_json = serde_json::to_string(&document.tags)
                .map_err(|_| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to serialize tags"))))?;
                
            conn.execute(
                "INSERT INTO documents (id, user_id, name, file_path, file_type, file_size, created_at, updated_at, tags) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    document.id,
                    document.user_id,
                    document.name,
                    document.file_path,
                    document.file_type,
                    document.file_size,
                    document.created_at.to_rfc3339(),
                    document.updated_at.to_rfc3339(),
                    tags_json
                ]
            )?;
            Ok(())
        })
    }
    
    pub fn get_documents_by_user(&self, user_id: &str) -> SqliteResult<Vec<Document>> {
        self.execute_with_retry(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, user_id, name, file_path, file_type, file_size, created_at, updated_at, tags FROM documents WHERE user_id = ?1 ORDER BY created_at DESC"
            )?;
            
            let document_iter = stmt.query_map([user_id], |row| {
                let created_at_str: String = row.get(6)?;
                let updated_at_str: String = row.get(7)?;
                let tags_json: String = row.get(8)?;
                
                let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
                
                Ok(Document {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    name: row.get(2)?,
                    file_path: row.get(3)?,
                    file_type: row.get(4)?,
                    file_size: row.get(5)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map_err(|_| rusqlite::Error::InvalidColumnType(6, "created_at".to_string(), rusqlite::types::Type::Text))?
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                        .map_err(|_| rusqlite::Error::InvalidColumnType(7, "updated_at".to_string(), rusqlite::types::Type::Text))?
                        .with_timezone(&Utc),
                    tags,
                })
            })?;
            
            let mut documents = Vec::new();
            for document in document_iter {
                documents.push(document?);
            }
            
            Ok(documents)
        })
    }
    
    pub fn log_activity(&self, activity: &Activity) -> SqliteResult<()> {
        self.execute_with_retry(|conn| {
            conn.execute(
                "INSERT INTO activities (id, user_id, action, resource_type, resource_id, details, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    activity.id,
                    activity.user_id,
                    activity.action,
                    activity.resource_type,
                    activity.resource_id,
                    activity.details,
                    activity.created_at.to_rfc3339()
                ]
            )?;
            Ok(())
        })
    }
    
    pub fn get_user_stats(&self, user_id: &str) -> SqliteResult<(i64, i64, i64)> {
        self.execute_with_retry(|conn| {
            // Total de documentos
            let mut stmt = conn.prepare("SELECT COUNT(*) FROM documents WHERE user_id = ?1")?;
            let document_count: i64 = stmt.query_row([user_id], |row| row.get(0))?;
            
            // Total de atividades
            let mut stmt = conn.prepare("SELECT COUNT(*) FROM activities WHERE user_id = ?1")?;
            let activity_count: i64 = stmt.query_row([user_id], |row| row.get(0))?;
            
            // Tamanho total dos arquivos
            let mut stmt = conn.prepare("SELECT COALESCE(SUM(file_size), 0) FROM documents WHERE user_id = ?1")?;
            let total_size: i64 = stmt.query_row([user_id], |row| row.get(0))?;
            
            Ok((document_count, activity_count, total_size))
        })
    }
    
    // COMANDO PARA VERIFICAR INTEGRIDADE DO BANCO
    pub fn verify_integrity(&self) -> SqliteResult<bool> {
        self.execute_with_retry(|conn| {
            let mut stmt = conn.prepare("PRAGMA integrity_check")?;
            let result: String = stmt.query_row([], |row| row.get(0))?;
            Ok(result == "ok")
        })
    }
    
    // OTIMIZAR BANCO (VACUUM + ANALYZE)
    pub fn optimize(&self) -> SqliteResult<()> {
        self.execute_with_retry(|conn| {
            conn.execute("PRAGMA optimize", [])?;
            conn.execute("ANALYZE", [])?;
            Ok(())
        })
    }
    
    // CHECKPOINT MANUAL DO WAL
    pub fn checkpoint(&self) -> SqliteResult<()> {
        self.execute_with_retry(|conn| {
            conn.execute("PRAGMA wal_checkpoint(TRUNCATE)", [])?;
            Ok(())
        })
    }
}
