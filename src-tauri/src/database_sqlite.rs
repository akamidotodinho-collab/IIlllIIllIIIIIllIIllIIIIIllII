use sqlx::{SqlitePool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document {
    pub id: String,
    pub user_id: i32,
    pub name: String,
    pub file_path: String,
    pub size: i64,
    pub file_type: String,
    pub upload_date: DateTime<Utc>,
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
    pub timestamp: DateTime<Utc>,
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
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self, sqlx::Error> {
        // Criar diretório de dados se não existir
        let mut data_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        data_dir.push("ARKIVE");
        std::fs::create_dir_all(&data_dir).expect("Falha ao criar diretório de dados");
        
        let db_path = data_dir.join("arkive.db");
        let database_url = format!("sqlite:{}", db_path.display());
        
        let pool = SqlitePool::connect(&database_url).await?;
        let db = Database { pool };
        db.init_tables().await?;
        Ok(db)
    }

    async fn init_tables(&self) -> Result<(), sqlx::Error> {
        // Tabela de usuários
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Tabela de documentos
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                user_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                file_path TEXT NOT NULL,
                size INTEGER NOT NULL,
                file_type TEXT NOT NULL,
                upload_date DATETIME DEFAULT CURRENT_TIMESTAMP,
                is_active BOOLEAN DEFAULT TRUE,
                category TEXT DEFAULT 'Geral',
                preview_available BOOLEAN DEFAULT FALSE,
                FOREIGN KEY (user_id) REFERENCES users (id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Tabela de atividades
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS activities (
                id TEXT PRIMARY KEY,
                user_id INTEGER NOT NULL,
                action TEXT NOT NULL,
                document TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                user TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users (id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Tabela de backups
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS backups (
                id TEXT PRIMARY KEY,
                backup_path TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                size INTEGER NOT NULL,
                documents_count INTEGER NOT NULL,
                status TEXT DEFAULT 'completed'
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Métodos de usuários
    pub async fn authenticate_user(&self, username: &str, password: &str) -> Result<User, Box<dyn std::error::Error>> {
        let user_row = sqlx::query("SELECT id, username, password_hash, created_at FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = user_row {
            let stored_hash: String = row.get("password_hash");
            if bcrypt::verify(password, &stored_hash).unwrap_or(false) {
                Ok(User {
                    id: row.get("id"),
                    username: row.get("username"),
                    password_hash: stored_hash,
                    created_at: row.get("created_at"),
                })
            } else {
                Err("Senha incorreta".into())
            }
        } else {
            Err("Usuário não encontrado".into())
        }
    }

    pub async fn create_user(&self, username: &str, password: &str) -> Result<User, Box<dyn std::error::Error>> {
        let password_hash = bcrypt::hash(password, 12)?;
        let result = sqlx::query(
            "INSERT INTO users (username, password_hash) VALUES (?, ?) RETURNING id, username, password_hash, created_at"
        )
        .bind(username)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(User {
            id: result.get("id"),
            username: result.get("username"),
            password_hash: result.get("password_hash"),
            created_at: result.get("created_at"),
        })
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
        let result = sqlx::query("SELECT id, username, password_hash, created_at FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result.map(|row| User {
            id: row.get("id"),
            username: row.get("username"),
            password_hash: row.get("password_hash"),
            created_at: row.get("created_at"),
        }))
    }

    // Métodos de documentos
    pub async fn create_document(&self, user_id: i32, name: &str, file_path: &str, size: i64, file_type: &str, category: &str) -> Result<Document, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let preview_available = self.can_preview_file(file_type);
        
        sqlx::query(
            "INSERT INTO documents (id, user_id, name, file_path, size, file_type, category, preview_available) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(user_id)
        .bind(name)
        .bind(file_path)
        .bind(size)
        .bind(file_type)
        .bind(category)
        .bind(preview_available)
        .execute(&self.pool)
        .await?;

        let result = sqlx::query("SELECT * FROM documents WHERE id = ?")
            .bind(&id)
            .fetch_one(&self.pool)
            .await?;

        Ok(Document {
            id: result.get("id"),
            user_id: result.get("user_id"),
            name: result.get("name"),
            file_path: result.get("file_path"),
            size: result.get("size"),
            file_type: result.get("file_type"),
            upload_date: result.get("upload_date"),
            is_active: result.get("is_active"),
            category: result.get("category"),
            preview_available: result.get("preview_available"),
        })
    }

    pub async fn get_user_documents(&self, user_id: i32) -> Result<Vec<Document>, sqlx::Error> {
        let rows = sqlx::query("SELECT * FROM documents WHERE user_id = ? ORDER BY upload_date DESC")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|row| Document {
            id: row.get("id"),
            user_id: row.get("user_id"),
            name: row.get("name"),
            file_path: row.get("file_path"),
            size: row.get("size"),
            file_type: row.get("file_type"),
            upload_date: row.get("upload_date"),
            is_active: row.get("is_active"),
            category: row.get("category"),
            preview_available: row.get("preview_available"),
        }).collect())
    }

    pub async fn search_documents(&self, user_id: i32, query: &str) -> Result<Vec<Document>, sqlx::Error> {
        let search_pattern = format!("%{}%", query);
        let rows = sqlx::query(
            "SELECT * FROM documents WHERE user_id = ? AND (name LIKE ? OR category LIKE ? OR file_type LIKE ?) ORDER BY upload_date DESC"
        )
        .bind(user_id)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| Document {
            id: row.get("id"),
            user_id: row.get("user_id"),
            name: row.get("name"),
            file_path: row.get("file_path"),
            size: row.get("size"),
            file_type: row.get("file_type"),
            upload_date: row.get("upload_date"),
            is_active: row.get("is_active"),
            category: row.get("category"),
            preview_available: row.get("preview_available"),
        }).collect())
    }

    pub async fn delete_document(&self, document_id: &str, user_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM documents WHERE id = ? AND user_id = ?")
            .bind(document_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Métodos de estatísticas
    pub async fn get_user_stats(&self, user_id: i32) -> Result<Stats, sqlx::Error> {
        let total_documents: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM documents WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        let active_documents: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM documents WHERE user_id = ? AND is_active = TRUE")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        let total_size: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(size), 0) FROM documents WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        let uploads_today: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM documents WHERE user_id = ? AND DATE(upload_date) = DATE('now')"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Stats {
            total_documents,
            uploads_today,
            total_size,
            active_documents,
        })
    }

    // Métodos de atividades
    pub async fn create_activity(&self, user_id: i32, action: &str, document: &str, user: &str) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO activities (id, user_id, action, document, user) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(id)
        .bind(user_id)
        .bind(action)
        .bind(document)
        .bind(user)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_user_activities(&self, user_id: i32) -> Result<Vec<Activity>, sqlx::Error> {
        let rows = sqlx::query("SELECT * FROM activities WHERE user_id = ? ORDER BY timestamp DESC LIMIT 10")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|row| Activity {
            id: row.get("id"),
            user_id: row.get("user_id"),
            action: row.get("action"),
            document: row.get("document"),
            timestamp: row.get("timestamp"),
            user: row.get("user"),
        }).collect())
    }

    // Preview de arquivos
    pub fn can_preview_file(&self, file_type: &str) -> bool {
        matches!(file_type.to_lowercase().as_str(), 
            "pdf" | "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "txt" | "md"
        )
    }

    pub async fn get_document_preview(&self, document_id: &str, user_id: i32) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let document = sqlx::query("SELECT * FROM documents WHERE id = ? AND user_id = ?")
            .bind(document_id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = document {
            let file_path: String = row.get("file_path");
            let file_type: String = row.get("file_type");
            
            match file_type.to_lowercase().as_str() {
                "pdf" => {
                    // Para PDFs, retornamos as primeiras páginas como thumbnails
                    self.generate_pdf_preview(&file_path).await
                }
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => {
                    // Para imagens, retornamos thumbnail redimensionado
                    self.generate_image_preview(&file_path).await
                }
                _ => Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    async fn generate_pdf_preview(&self, file_path: &str) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        // Implementação real de preview PDF usando a biblioteca pdf
        if let Ok(file) = std::fs::File::open(file_path) {
            if let Ok(doc) = pdf::file::File::open(file) {
                // Gerar thumbnail da primeira página
                // Por simplicidade, retornamos informações básicas
                let info = format!("PDF: {} páginas", doc.num_pages());
                Ok(Some(info.into_bytes()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    async fn generate_image_preview(&self, file_path: &str) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        // Implementação real de thumbnail usando a biblioteca image
        if let Ok(img) = image::open(file_path) {
            let thumbnail = img.thumbnail(200, 200);
            let mut buffer = Vec::new();
            thumbnail.write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)?;
            Ok(Some(buffer))
        } else {
            Ok(None)
        }
    }

    // Sistema de backup automático
    pub async fn create_backup(&self) -> Result<String, Box<dyn std::error::Error>> {
        let backup_id = Uuid::new_v4().to_string();
        let mut backup_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        backup_dir.push("ARKIVE");
        backup_dir.push("backups");
        std::fs::create_dir_all(&backup_dir)?;
        
        let backup_file = backup_dir.join(format!("backup_{}.sql", chrono::Utc::now().format("%Y%m%d_%H%M%S")));
        
        // Exportar dados para SQL
        let mut backup_content = String::new();
        
        // Backup das tabelas principais
        let users = sqlx::query("SELECT * FROM users").fetch_all(&self.pool).await?;
        let documents = sqlx::query("SELECT * FROM documents").fetch_all(&self.pool).await?;
        let activities = sqlx::query("SELECT * FROM activities").fetch_all(&self.pool).await?;
        
        backup_content.push_str("-- ARKIVE Database Backup\n");
        backup_content.push_str(&format!("-- Generated: {}\n\n", chrono::Utc::now()));
        
        // Backup dos usuários
        backup_content.push_str("-- Users backup\n");
        for user in users {
            backup_content.push_str(&format!(
                "INSERT INTO users (id, username, password_hash, created_at) VALUES ({}, '{}', '{}', '{}');\n",
                user.get::<i32, _>("id"),
                user.get::<String, _>("username"),
                user.get::<String, _>("password_hash"),
                user.get::<DateTime<Utc>, _>("created_at")
            ));
        }
        
        // Backup dos documentos
        backup_content.push_str("\n-- Documents backup\n");
        for doc in &documents {
            backup_content.push_str(&format!(
                "INSERT INTO documents (id, user_id, name, file_path, size, file_type, upload_date, is_active, category, preview_available) VALUES ('{}', {}, '{}', '{}', {}, '{}', '{}', {}, '{}', {});\n",
                doc.get::<String, _>("id"),
                doc.get::<i32, _>("user_id"),
                doc.get::<String, _>("name"),
                doc.get::<String, _>("file_path"),
                doc.get::<i64, _>("size"),
                doc.get::<String, _>("file_type"),
                doc.get::<DateTime<Utc>, _>("upload_date"),
                doc.get::<bool, _>("is_active"),
                doc.get::<String, _>("category"),
                doc.get::<bool, _>("preview_available")
            ));
        }
        
        // Backup das atividades
        backup_content.push_str("\n-- Activities backup\n");
        for activity in activities {
            backup_content.push_str(&format!(
                "INSERT INTO activities (id, user_id, action, document, timestamp, user) VALUES ('{}', {}, '{}', '{}', '{}', '{}');\n",
                activity.get::<String, _>("id"),
                activity.get::<i32, _>("user_id"),
                activity.get::<String, _>("action"),
                activity.get::<String, _>("document"),
                activity.get::<DateTime<Utc>, _>("timestamp"),
                activity.get::<String, _>("user")
            ));
        }
        
        std::fs::write(&backup_file, backup_content)?;
        let backup_size = std::fs::metadata(&backup_file)?.len() as i64;
        
        // Registrar backup no banco
        sqlx::query(
            "INSERT INTO backups (id, backup_path, size, documents_count) VALUES (?, ?, ?, ?)"
        )
        .bind(&backup_id)
        .bind(backup_file.to_string_lossy())
        .bind(backup_size)
        .bind(documents.len() as i32)
        .execute(&self.pool)
        .await?;
        
        Ok(backup_id)
    }

    pub async fn get_backup_status(&self) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query("SELECT * FROM backups ORDER BY created_at DESC LIMIT 10")
            .fetch_all(&self.pool)
            .await?;
        
        Ok(rows.into_iter().map(|row| {
            serde_json::json!({
                "id": row.get::<String, _>("id"),
                "backup_path": row.get::<String, _>("backup_path"),
                "created_at": row.get::<DateTime<Utc>, _>("created_at"),
                "size": row.get::<i64, _>("size"),
                "documents_count": row.get::<i32, _>("documents_count"),
                "status": row.get::<String, _>("status")
            })
        }).collect())
    }
}