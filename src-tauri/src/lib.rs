use tauri::Manager;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub id: i64,
    pub name: String,
    pub file_type: String,
    pub size: i64,
    pub upload_date: String,
    pub user_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Activity {
    pub id: i64,
    pub activity_type: String,
    pub description: String,
    pub timestamp: String,
    pub user_id: i64,
}

// Comandos Tauri simplificados - sem threading issues
#[tauri::command]
fn login(request: LoginRequest) -> Result<User, String> {
    // Versão simplificada para compilação
    Ok(User {
        id: 1,
        username: request.username,
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

#[tauri::command]
fn register(request: RegisterRequest) -> Result<User, String> {
    Ok(User {
        id: 1,
        username: request.username,
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

#[tauri::command]
fn get_current_user() -> Result<Option<User>, String> {
    Ok(None)
}

#[tauri::command]
fn logout() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn get_dashboard_stats() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "total_documents": 0,
        "active_documents": 0,
        "total_size": 0,
        "uploads_today": 0
    }))
}

#[tauri::command]
fn get_recent_activities() -> Result<Vec<Activity>, String> {
    Ok(vec![])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            login,
            register,
            get_current_user,
            logout,
            get_dashboard_stats,
            get_recent_activities
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
