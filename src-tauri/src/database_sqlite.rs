use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use dirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub name: String,
    pub size: i64,
    pub file_type: String,
    pub upload_date: String,
    pub is_active: bool,
    pub category: String,
    pub preview_available: bool,
    pub user_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub action: String,
    pub document: String,
    pub timestamp: String,
    pub user: String,
    pub user_id: i64,
}

#[derive(Debug, Clone)]
pub struct UserStats {
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
        log::info!("🗄️ Inicializando banco de dados SQLite...");
        
        let app_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("arkive");
            
        if !app_dir.exists() {
            std::fs::create_dir_all(&app_dir)?;
        }
        
        let db_path = app_dir.join("arkive.db");
        log::info!("📍 Caminho do banco: {:?}", db_path);
        
        let conn = Connection::open(db_path)?;
        
        let db = Database { conn };
        db.init_tables()?;
        
        log::info!("✅ Banco de dados inicializado com sucesso");
        Ok(db)
    }
    
    fn init_tables(&self) -> Result<(), rusqlite::Error> {
        log::info!("📋 Criando tabelas do banco...");
        
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                size INTEGER NOT NULL,
                file_type TEXT NOT NULL,
                upload_date DATETIME DEFAULT CURRENT_TIMESTAMP,
                is_active BOOLEAN DEFAULT 1,
                category TEXT DEFAULT 'Geral',
                preview_available BOOLEAN DEFAULT 0,
                user_id INTEGER NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users (id)
            )",
            [],
        )?;
        
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS activities (
                id TEXT PRIMARY KEY,
                action TEXT NOT NULL,
                document TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                user TEXT NOT NULL,
                user_id INTEGER NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users (id)
            )",
            [],
        )?;
        
        log::info!("✅ Tabelas criadas com sucesso");
        Ok(())
    }
    
    pub async fn create_user(&self, username: &str, password: &str) -> Result<i64, Box<dyn std::error::Error>> {
        log::info!("👤 Criando usuário: {}", username);
        
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;
        
        let user_id = self.conn.execute(
            "INSERT INTO users (username, password_hash) VALUES (?1, ?2)",
            [username, &password_hash],
        )? as i64;
        
        log::info!("✅ Usuário criado com ID: {}", user_id);
        Ok(user_id)
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
            Err(e) => Err(e.into()),
        }
    }
    
    pub async fn authenticate_user(&self, username: &str, password: &str) -> Result<Option<User>, Box<dyn std::error::Error>> {
        if let Some(user) = self.get_user_by_username(username).await? {
            if bcrypt::verify(password, &user.password_hash)? {
                log::info!("✅ Usuário autenticado: {}", username);
                return Ok(Some(user));
            }
        }
        
        log::warn!("❌ Falha na autenticação: {}", username);
        Ok(None)
    }
    
    pub async fn get_user_stats(&self, user_id: i64) -> Result<UserStats, Box<dyn std::error::Error>> {
        let total_documents: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM documents WHERE user_id = ?1",
            [user_id],
            |row| row.get(0),
        ).unwrap_or(0);
        
        let active_documents: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM documents WHERE user_id = ?1 AND is_active = 1",
            [user_id],
            |row| row.get(0),
        ).unwrap_or(0);
        
        let total_size: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(size), 0) FROM documents WHERE user_id = ?1",
            [user_id],
            |row| row.get(0),
        ).unwrap_or(0);
        
        let uploads_today: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM documents WHERE user_id = ?1 AND DATE(upload_date) = DATE('now')",
            [user_id],
            |row| row.get(0),
        ).unwrap_or(0);
        
        Ok(UserStats {
            total_documents,
            uploads_today,
            total_size,
            active_documents,
        })
    }
    
    pub async fn get_user_documents(&self, user_id: i64) -> Result<Vec<Document>, Box<dyn std::error::Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, size, file_type, upload_date, is_active, category, preview_available, user_id 
             FROM documents WHERE user_id = ?1 ORDER BY upload_date DESC"
        )?;
        
        let document_iter = stmt.query_map([user_id], |row| {
            Ok(Document {
                id: row.get(0)?,
                name: row.get(1)?,
                size: row.get(2)?,
                file_type: row.get(3)?,
                upload_date: row.get(4)?,
                is_active: row.get(5)?,
                category: row.get(6)?,
                preview_available: row.get(7)?,
                user_id: row.get(8)?,
            })
        })?;
        
        let mut documents = Vec::new();
        for doc in document_iter {
            documents.push(doc?);
        }
        
        Ok(documents)
    }
    
    pub async fn get_user_activities(&self, user_id: i64) -> Result<Vec<Activity>, Box<dyn std::error::Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, action, document, timestamp, user, user_id 
             FROM activities WHERE user_id = ?1 ORDER BY timestamp DESC LIMIT 50"
        )?;
        
        let activity_iter = stmt.query_map([user_id], |row| {
            Ok(Activity {
                id: row.get(0)?,
                action: row.get(1)?,
                document: row.get(2)?,
                timestamp: row.get(3)?,
                user: row.get(4)?,
                user_id: row.get(5)?,
            })
        })?;
        
        let mut activities = Vec::new();
        for activity in activity_iter {
            activities.push(activity?);
        }
        
        Ok(activities)
    }
}
