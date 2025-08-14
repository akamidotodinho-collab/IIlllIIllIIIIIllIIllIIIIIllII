use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

mod database_sqlite;
use database_sqlite::{Database, User};

// Estado da aplicação
pub struct AppState {
    pub db: Arc<Database>,
    pub authenticated_user: Arc<Mutex<Option<User>>>,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let db = Arc::new(Database::new().await?);
        let authenticated_user = Arc::new(Mutex::new(None));
        
        Ok(AppState {
            db,
            authenticated_user,
        })
    }
}

// Estruturas para responses da API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentResponse {
    pub id: String,
    pub name: String,
    pub size: i64,
    pub file_type: String,
    pub upload_date: String,
    pub is_active: bool,
    pub category: String,
    pub preview_available: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActivityResponse {
    pub id: String,
    pub action: String,
    pub document: String,
    pub timestamp: String,
    pub user: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsResponse {
    pub total_documents: i64,
    pub uploads_today: i64,
    pub total_size: String,
    pub active_documents: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub user: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub confirm_password: String,
}

// Comandos Tauri 2.2
#[tauri::command]
async fn login(
    request: LoginRequest,
    state: State<'_, AppState>,
) -> Result<LoginResponse, String> {
    let user_result = state.db.authenticate_user(&request.username, &request.password).await;
    
    match user_result {
        Ok(user) => {
            let mut authenticated_user = state.authenticated_user.lock().await;
            *authenticated_user = Some(user.clone());
            
            Ok(LoginResponse {
                success: true,
                user: Some(user.username),
                error: None,
            })
        }
        Err(e) => {
            Ok(LoginResponse {
                success: false,
                user: None,
                error: Some(format!("Login falhou: {}", e)),
            })
        }
    }
}

#[tauri::command]
async fn register(
    request: RegisterRequest,
    state: State<'_, AppState>,
) -> Result<LoginResponse, String> {
    if request.password != request.confirm_password {
        return Ok(LoginResponse {
            success: false,
            user: None,
            error: Some("Senhas não conferem".to_string()),
        });
    }

    if request.password.len() < 4 {
        return Ok(LoginResponse {
            success: false,
            user: None,
            error: Some("Senha deve ter pelo menos 4 caracteres".to_string()),
        });
    }

    let result = state.db.create_user(&request.username, &request.password).await;
    
    match result {
        Ok(user) => {
            let mut authenticated_user = state.authenticated_user.lock().await;
            *authenticated_user = Some(user.clone());
            
            Ok(LoginResponse {
                success: true,
                user: Some(user.username),
                error: None,
            })
        }
        Err(e) => {
            Ok(LoginResponse {
                success: false,
                user: None,
                error: Some(format!("Erro ao criar usuário: {}", e)),
            })
        }
    }
}

#[tauri::command]
async fn get_current_user(
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let authenticated_user = state.authenticated_user.lock().await;
    Ok(authenticated_user.as_ref().map(|user| user.username.clone()))
}

#[tauri::command]
async fn logout(
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let mut authenticated_user = state.authenticated_user.lock().await;
    *authenticated_user = None;
    Ok(true)
}

#[tauri::command]
async fn get_stats(
    state: State<'_, AppState>,
) -> Result<StatsResponse, String> {
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(user) = authenticated_user.as_ref() {
        let stats = state.db.get_user_stats(user.id).await
            .map_err(|e| format!("Erro ao buscar estatísticas: {}", e))?;
        Ok(StatsResponse {
            total_documents: stats.total_documents,
            uploads_today: stats.uploads_today,
            total_size: format!("{:.2} MB", stats.total_size as f64 / 1024.0 / 1024.0),
            active_documents: stats.active_documents,
        })
    } else {
        Err("Usuário não autenticado".to_string())
    }
}

#[tauri::command]
async fn get_documents(
    state: State<'_, AppState>,
) -> Result<Vec<DocumentResponse>, String> {
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(user) = authenticated_user.as_ref() {
        let documents = state.db.get_user_documents(user.id).await
            .map_err(|e| format!("Erro ao buscar documentos: {}", e))?;
        
        let doc_responses: Vec<DocumentResponse> = documents.into_iter().map(|doc| DocumentResponse {
            id: doc.id,
            name: doc.name,
            size: doc.size,
            file_type: doc.file_type,
            upload_date: doc.upload_date,
            is_active: doc.is_active,
            category: doc.category,
            preview_available: doc.preview_available,
        }).collect();
        
        Ok(doc_responses)
    } else {
        Err("Usuário não autenticado".to_string())
    }
}

#[tauri::command]
async fn get_recent_activities(
    state: State<'_, AppState>,
) -> Result<Vec<ActivityResponse>, String> {
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(user) = authenticated_user.as_ref() {
        let activities = state.db.get_user_activities(user.id).await
            .map_err(|e| format!("Erro ao buscar atividades: {}", e))?;
        
        let activity_responses: Vec<ActivityResponse> = activities.into_iter().map(|act| ActivityResponse {
            id: act.id,
            action: act.action,
            document: act.document,
            timestamp: act.timestamp,
            user: act.user,
        }).collect();
        
        Ok(activity_responses)
    } else {
        Err("Usuário não autenticado".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let app_state = rt.block_on(AppState::new()).expect("Failed to initialize app state");

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            login,
            register,
            get_current_user,
            logout,
            get_stats,
            get_documents,
            get_recent_activities
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}