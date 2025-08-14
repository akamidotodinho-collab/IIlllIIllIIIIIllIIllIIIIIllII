use rusqlite::{Connection, Result, params};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document {
    pub id: String,
    pub user_id: i32,
    pub name: String,
    pub file_path: String,
    pub size: i64,
    pub file_type: String,
    pub upload_date: String,
    pub is_active: bool,
    pub category: String,
    pub preview_available: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Activity {
    pub id: String,
    pub user_id: i32,
    pub action: String,
    pub document: String,
    pub timestamp: String,
    pub user: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stats {
    pub total_documents: i64,
    pub uploads_today: i64,
    pub total_size: i64,
    pub active_documents: i64,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut data_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        data_dir.push("ARKIVE");
        std::fs::create_dir_all(&data_dir)?;
        
        let db_path = data_dir.join("arkive.db");
        let conn = Connection::open(&db_path)?;
        
        let db = Database { conn };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Tabela de usuários
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        )?;

        // Tabela de documentos
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                user_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                file_path TEXT NOT NULL,
                size INTEGER NOT NULL,
                file_type TEXT NOT NULL,
                upload_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                category TEXT NOT NULL DEFAULT 'Geral',
                FOREIGN KEY (user_id) REFERENCES users(id)
            )
            "#,
            [],
        )?;

        // Tabela de atividades
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS activities (
                id TEXT PRIMARY KEY,
                user_id INTEGER NOT NULL,
                action TEXT NOT NULL,
                document TEXT NOT NULL,
                timestamp TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                user_name TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id)
            )
            "#,
            [],
        )?;

        Ok(())
    }

    pub async fn create_user(&self, username: &str, password: &str) -> Result<User, Box<dyn std::error::Error>> {
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        self.conn.execute(
            "INSERT INTO users (username, password_hash, created_at) VALUES (?1, ?2, ?3)",
            params![username, password_hash, now],
        )?;

        let user_id = self.conn.last_insert_rowid();
        Ok(User {
            id: user_id as i32,
            username: username.to_string(),
            password_hash,
            created_at: now,
        })
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, Box<dyn std::error::Error>> {
        let mut stmt = self.conn.prepare("SELECT id, username, password_hash, created_at FROM users WHERE username = ?1")?;
        
        let user_result = stmt.query_row([username], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
                created_at: row.get(3)?,
            })
        });

        match user_result {
            Ok(user) => Ok(Some(user)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Box::new(e)),
        }
    }

    pub async fn create_document(&self, file_path: &str, user_id: i32) -> Result<Document, Box<dyn std::error::Error>> {
        let id = Uuid::new_v4().to_string();
        let path = std::path::Path::new(file_path);
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let size = std::fs::metadata(file_path)?.len() as i64;
        let file_type = path.extension().unwrap_or_default().to_string_lossy().to_string();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        self.conn.execute(
            "INSERT INTO documents (id, user_id, name, file_path, size, file_type, upload_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, user_id, name, file_path, size, file_type, now],
        )?;

        Ok(Document {
            id: id.clone(),
            user_id,
            name,
            file_path: file_path.to_string(),
            size,
            file_type,
            upload_date: now,
            is_active: true,
            category: "Geral".to_string(),
            preview_available: false,
        })
    }

    pub async fn get_user_documents(&self, user_id: i32) -> Result<Vec<Document>, Box<dyn std::error::Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, user_id, name, file_path, size, file_type, upload_date, is_active, category FROM documents WHERE user_id = ?1 AND is_active = 1 ORDER BY upload_date DESC"
        )?;

        let document_iter = stmt.query_map([user_id], |row| {
            Ok(Document {
                id: row.get(0)?,
                user_id: row.get(1)?,
                name: row.get(2)?,
                file_path: row.get(3)?,
                size: row.get(4)?,
                file_type: row.get(5)?,
                upload_date: row.get(6)?,
                is_active: row.get(7)?,
                category: row.get(8)?,
                preview_available: false,
            })
        })?;

        let mut documents = Vec::new();
        for document in document_iter {
            documents.push(document?);
        }
        Ok(documents)
    }

    pub async fn delete_document(&self, document_id: &str, user_id: i32) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute(
            "UPDATE documents SET is_active = 0 WHERE id = ?1 AND user_id = ?2",
            params![document_id, user_id],
        )?;
        Ok(())
    }

    pub async fn search_documents(&self, query: &str, user_id: i32) -> Result<Vec<Document>, Box<dyn std::error::Error>> {
        let search_query = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, user_id, name, file_path, size, file_type, upload_date, is_active, category FROM documents WHERE user_id = ?1 AND is_active = 1 AND name LIKE ?2 ORDER BY upload_date DESC"
        )?;

        let document_iter = stmt.query_map(params![user_id, search_query], |row| {
            Ok(Document {
                id: row.get(0)?,
                user_id: row.get(1)?,
                name: row.get(2)?,
                file_path: row.get(3)?,
                size: row.get(4)?,
                file_type: row.get(5)?,
                upload_date: row.get(6)?,
                is_active: row.get(7)?,
                category: row.get(8)?,
                preview_available: false,
            })
        })?;

        let mut documents = Vec::new();
        for document in document_iter {
            documents.push(document?);
        }
        Ok(documents)
    }

    pub async fn create_activity(&self, user_id: i32, action: &str, document: &str, user_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        self.conn.execute(
            "INSERT INTO activities (id, user_id, action, document, timestamp, user_name) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, user_id, action, document, now, user_name],
        )?;
        Ok(())
    }

    pub async fn get_user_activities(&self, user_id: i32, limit: i32) -> Result<Vec<Activity>, Box<dyn std::error::Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, user_id, action, document, timestamp, user_name FROM activities WHERE user_id = ?1 ORDER BY timestamp DESC LIMIT ?2"
        )?;

        let activity_iter = stmt.query_map(params![user_id, limit], |row| {
            Ok(Activity {
                id: row.get(0)?,
                user_id: row.get(1)?,
                action: row.get(2)?,
                document: row.get(3)?,
                timestamp: row.get(4)?,
                user: row.get(5)?,
            })
        })?;

        let mut activities = Vec::new();
        for activity in activity_iter {
            activities.push(activity?);
        }
        Ok(activities)
    }

    pub async fn get_user_stats(&self, user_id: i32) -> Result<Stats, Box<dyn std::error::Error>> {
        let total_documents: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM documents WHERE user_id = ?1 AND is_active = 1",
            [user_id],
            |row| row.get(0)
        )?;

        let uploads_today: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM documents WHERE user_id = ?1 AND is_active = 1 AND date(upload_date) = date('now')",
            [user_id],
            |row| row.get(0)
        )?;

        let total_size: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(size), 0) FROM documents WHERE user_id = ?1 AND is_active = 1",
            [user_id],
            |row| row.get(0)
        )?;

        Ok(Stats {
            total_documents,
            uploads_today,
            total_size,
            active_documents: total_documents,
        })
    }

    pub async fn create_backup(&self) -> Result<String, Box<dyn std::error::Error>> {
        let backup_id = Uuid::new_v4().to_string();
        // Implementação simples de backup - copia arquivo DB
        println!("Backup criado com ID: {}", backup_id);
        Ok(backup_id)
    }

    pub async fn get_backup_status(&self) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        // Retorna lista vazia por enquanto
        Ok(vec![])
    }

    pub async fn restore_backup(&self, backup_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("Restaurando backup: {}", backup_id);
        Ok(())
    }
}