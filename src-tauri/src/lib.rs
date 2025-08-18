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
        log::info!("🔧 Inicializando AppState...");
        
        log::info!("📊 Conectando ao banco de dados...");
        let db = match Database::new().await {
            Ok(database) => {
                log::info!("✅ Banco de dados conectado com sucesso");
                Arc::new(database)
            },
            Err(e) => {
                log::error!("❌ Erro ao conectar banco de dados: {:?}", e);
                return Err(e.into());
            }
        };
        
        let authenticated_user = Arc::new(Mutex::new(None));
        log::info!("✅ AppState inicializado com sucesso");
        
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

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub message: String,
}

// Comandos Tauri
#[tauri::command]
async fn login(
    request: LoginRequest,
    state: State<'_, AppState>,
) -> Result<LoginResponse, String> {
    if request.username.is_empty() || request.password.is_empty() {
        return Ok(LoginResponse {
            success: false,
            user: None,
            error: Some("Usuário e senha são obrigatórios".to_string()),
        });
    }

    match state.db.authenticate_user(&request.username, &request.password).await {
        Ok(Some(user)) => {
            let mut authenticated_user = state.authenticated_user.lock().await;
            *authenticated_user = Some(user.clone());
            
            Ok(LoginResponse {
                success: true,
                user: Some(user.username),
                error: None,
            })
        },
        Ok(None) => Ok(LoginResponse {
            success: false,
            user: None,
            error: Some("Credenciais inválidas".to_string()),
        }),
        Err(e) => Ok(LoginResponse {
            success: false,
            user: None,
            error: Some(format!("Erro interno: {}", e)),
        }),
    }
}

#[tauri::command]
async fn register(
    request: RegisterRequest,
    state: State<'_, AppState>,
) -> Result<RegisterResponse, String> {
    // Validações básicas
    if request.username.is_empty() || request.password.is_empty() {
        return Ok(RegisterResponse {
            success: false,
            message: "Usuário e senha são obrigatórios".to_string(),
        });
    }

    if request.password != request.confirm_password {
        return Ok(RegisterResponse {
            success: false,
            message: "Senhas não conferem".to_string(),
        });
    }

    if request.password.len() < 6 {
        return Ok(RegisterResponse {
            success: false,
            message: "Senha deve ter pelo menos 6 caracteres".to_string(),
        });
    }

    // Verificar se usuário já existe
    match state.db.get_user_by_username(&request.username).await {
        Ok(Some(_)) => Ok(RegisterResponse {
            success: false,
            message: "Usuário já existe".to_string(),
        }),
        Ok(None) => {
            // Criar novo usuário
            match state.db.create_user(&request.username, &request.password).await {
                Ok(_) => Ok(RegisterResponse {
                    success: true,
                    message: "Usuário criado com sucesso".to_string(),
                }),
                Err(e) => Ok(RegisterResponse {
                    success: false,
                    message: format!("Erro ao criar usuário: {}", e),
                }),
            }
        },
        Err(e) => Ok(RegisterResponse {
            success: false,
            message: format!("Erro interno: {}", e),
        }),
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
) -> Result<(), String> {
    let mut authenticated_user = state.authenticated_user.lock().await;
    *authenticated_user = None;
    Ok(())
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
    log::info!("🏃 Iniciando função run()...");
    
    log::info!("⚙️ Criando runtime Tokio...");
    let rt = match tokio::runtime::Runtime::new() {
        Ok(runtime) => {
            log::info!("✅ Runtime Tokio criado com sucesso");
            runtime
        },
        Err(e) => {
            log::error!("❌ Falha ao criar runtime Tokio: {:?}", e);
            panic!("Failed to create Tokio runtime: {:?}", e);
        }
    };
    
    log::info!("🔧 Inicializando AppState...");
    let app_state = match rt.block_on(AppState::new()) {
        Ok(state) => {
            log::info!("✅ AppState inicializado com sucesso");
            state
        },
        Err(e) => {
            log::error!("❌ Falha ao inicializar AppState: {:?}", e);
            panic!("Failed to initialize app state: {:?}", e);
        }
    };

    log::info!("🚀 Iniciando aplicação Tauri...");
    if let Err(e) = tauri::Builder::default()
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
    {
        log::error!("❌ Erro ao executar aplicação Tauri: {:?}", e);
        panic!("Error while running tauri application: {:?}", e);
    }
}
