use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;

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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Storage {
    pub users: HashMap<i32, User>,
    pub documents: HashMap<String, Document>,
    pub activities: Vec<Activity>,
    pub next_user_id: i32,
}

impl Storage {
    pub fn new() -> Self {
        let storage_path = Self::get_storage_path();
        if storage_path.exists() {
            match fs::read_to_string(&storage_path) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(storage) => storage,
                        Err(_) => Self::default(),
                    }
                }
                Err(_) => Self::default(),
            }
        } else {
            let mut storage = Self::default();
            storage.next_user_id = 1;
            storage
        }
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let storage_path = Self::get_storage_path();
        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&storage_path, content)?;
        Ok(())
    }

    fn get_storage_path() -> PathBuf {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("ARKIVE");
        path.push("storage.json");
        path
    }

    pub fn create_user(&mut self, username: &str, password_hash: &str) -> Result<User, String> {
        // Verificar se usuário já existe
        for user in self.users.values() {
            if user.username == username {
                return Err("Usuário já existe".to_string());
            }
        }

        let user = User {
            id: self.next_user_id,
            username: username.to_string(),
            password_hash: password_hash.to_string(),
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        self.users.insert(self.next_user_id, user.clone());
        self.next_user_id += 1;
        self.save().map_err(|e| format!("Erro ao salvar: {}", e))?;

        Ok(user)
    }

    pub fn get_user_by_username(&self, username: &str) -> Option<User> {
        self.users.values()
            .find(|user| user.username == username)
            .cloned()
    }

    pub fn get_user_stats(&self, user_id: i32) -> Stats {
        let user_docs: Vec<&Document> = self.documents.values()
            .filter(|doc| doc.user_id == user_id)
            .collect();

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let uploads_today = user_docs.iter()
            .filter(|doc| doc.upload_date.starts_with(&today))
            .count() as i64;

        Stats {
            total_documents: user_docs.len() as i64,
            uploads_today,
            total_size: user_docs.iter().map(|doc| doc.size).sum(),
            active_documents: user_docs.iter().filter(|doc| doc.is_active).count() as i64,
        }
    }

    pub fn get_user_documents(&self, user_id: i32) -> Vec<Document> {
        self.documents.values()
            .filter(|doc| doc.user_id == user_id)
            .cloned()
            .collect()
    }

    pub fn get_user_activities(&self, user_id: i32, limit: usize) -> Vec<Activity> {
        self.activities.iter()
            .filter(|activity| activity.user_id == user_id)
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn create_activity(&mut self, user_id: i32, action: &str, document: &str, user: &str) -> Result<(), String> {
        let activity = Activity {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            action: action.to_string(),
            document: document.to_string(),
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            user: user.to_string(),
        };

        self.activities.insert(0, activity);
        // Manter apenas as últimas 1000 atividades
        if self.activities.len() > 1000 {
            self.activities.truncate(1000);
        }

        self.save().map_err(|e| format!("Erro ao salvar: {}", e))?;
        Ok(())
    }

    pub fn search_documents(&self, user_id: i32, query: &str) -> Result<Vec<Document>, String> {
        let query_lower = query.to_lowercase();
        let documents: Vec<Document> = self.documents.values()
            .filter(|doc| {
                doc.user_id == user_id && (
                    doc.name.to_lowercase().contains(&query_lower) ||
                    doc.category.to_lowercase().contains(&query_lower) ||
                    doc.file_type.to_lowercase().contains(&query_lower)
                )
            })
            .cloned()
            .collect();
        Ok(documents)
    }

    pub fn create_document(&mut self, user_id: i32, name: &str, file_path: &str, size: i64, file_type: &str, category: &str) -> Result<Document, String> {
        let document = Document {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            name: name.to_string(),
            file_path: file_path.to_string(),
            size,
            file_type: file_type.to_string(),
            upload_date: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            is_active: true,
            category: category.to_string(),
        };

        self.documents.insert(document.id.clone(), document.clone());
        self.save().map_err(|e| format!("Erro ao salvar: {}", e))?;
        Ok(document)
    }
}