use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

mod database_sqlite;
mod backup;

use database_sqlite::{Database, User};
use std::path::PathBuf;

// Estado da aplicação
pub struct AppState {
    pub db: Arc<Database>,
    pub authenticated_user: Arc<Mutex<Option<User>>>,
}

impl AppState {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        log::info!("🔧 Inicializando AppState...");
        
        log::info!("📊 Conectando ao banco de dados...");
        
        // Criar diretório de dados
        let mut data_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        data_dir.push("ARKIVE");
        std::fs::create_dir_all(&data_dir)?;
        
        let db_path = data_dir.join("arkive.db");
        
        let db = match Database::new(db_path) {
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

// Comandos Tauri básicos (implementação mínima)
#[tauri::command]
async fn login(
    request: LoginRequest,
    state: State<'_, AppState>,
) -> Result<LoginResponse, String> {
    // Implementação simplificada para compatibilidade
    let user_result = state.db.get_user_by_username(&request.username);
    
    match user_result {
        Ok(Some(user)) => {
            if bcrypt::verify(&request.password, &user.password_hash).unwrap_or(false) {
                let mut authenticated_user = state.authenticated_user.lock().await;
                *authenticated_user = Some(user.clone());
                
                Ok(LoginResponse {
                    success: true,
                    user: Some(user.username),
                    error: None,
                })
            } else {
                Ok(LoginResponse {
                    success: false,
                    user: None,
                    error: Some("Senha incorreta".to_string()),
                })
            }
        }
        Ok(None) => {
            Ok(LoginResponse {
                success: false,
                user: None,
                error: Some("Usuário não encontrado".to_string()),
            })
        }
        Err(e) => {
            Ok(LoginResponse {
                success: false,
                user: None,
                error: Some(format!("Erro de banco: {:?}", e)),
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

    // Criar usuário
    let password_hash = match bcrypt::hash(&request.password, 12) {
        Ok(hash) => hash,
        Err(e) => return Ok(LoginResponse {
            success: false,
            user: None,
            error: Some(format!("Erro ao criptografar senha: {}", e)),
        }),
    };

    let new_user = User {
        id: uuid::Uuid::new_v4().to_string(),
        username: request.username.clone(),
        email: format!("{}@arkive.local", request.username), // Email padrão
        password_hash,
        created_at: chrono::Utc::now(),
        last_login: None,
    };

    match state.db.create_user(&new_user) {
        Ok(_) => {
            let mut authenticated_user = state.authenticated_user.lock().await;
            *authenticated_user = Some(new_user.clone());
            
            Ok(LoginResponse {
                success: true,
                user: Some(new_user.username),
                error: None,
            })
        }
        Err(e) => {
            Ok(LoginResponse {
                success: false,
                user: None,
                error: Some(format!("Erro ao criar usuário: {:?}", e)),
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
        let stats = state.db.get_user_stats(&user.id)
            .map_err(|e| format!("Erro ao buscar estatísticas: {:?}", e))?;
        
        Ok(StatsResponse {
            total_documents: stats.0,
            uploads_today: stats.1, // Usar total de atividades como proxy
            total_size: format_size(stats.2),
            active_documents: stats.0, // Assumir todos documentos são ativos
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
        let documents = state.db.get_documents_by_user(&user.id)
            .map_err(|e| format!("Erro ao buscar documentos: {:?}", e))?;
        
        let response: Vec<DocumentResponse> = documents.into_iter().map(|doc| {
            DocumentResponse {
                id: doc.id,
                name: doc.name,
                size: doc.file_size,
                file_type: doc.file_type,
                upload_date: doc.created_at.format("%d/%m/%Y").to_string(),
                is_active: true,
                category: "Documento".to_string(),
                preview_available: false,
            }
        }).collect();
        
        Ok(response)
    } else {
        Err("Usuário não autenticado".to_string())
    }
}

#[tauri::command]
async fn get_recent_activities(
    state: State<'_, AppState>,
) -> Result<Vec<ActivityResponse>, String> {
    // Implementação simplificada retornando atividades fictícias
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(user) = authenticated_user.as_ref() {
        Ok(vec![
            ActivityResponse {
                id: "1".to_string(),
                action: "Login realizado".to_string(),
                document: "Sistema".to_string(),
                timestamp: chrono::Utc::now().format("%H:%M").to_string(),
                user: user.username.clone(),
            }
        ])
    } else {
        Err("Usuário não autenticado".to_string())
    }
}

// Função utilitária para formatar tamanho
fn format_size(bytes: i64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    format!("{:.1} {}", size, UNITS[unit_index])
}

// Função principal do Tauri
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Configurar logs
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    log::info!("🚀 Iniciando ARKIVE Desktop...");
    
    // Inicializar runtime Tokio
    let rt = match tokio::runtime::Runtime::new() {
        Ok(runtime) => {
            log::info!("✅ Tokio runtime inicializado");
            runtime
        },
        Err(e) => {
            log::error!("❌ Falha ao criar Tokio runtime: {:?}", e);
            panic!("Failed to create Tokio runtime: {:?}", e);
        }
    };
    
    log::info!("🔧 Inicializando AppState...");
    let app_state = match AppState::new() {
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
            get_recent_activities,
            backup::verify_backup_file,
            backup::list_available_backups,
        ])
        .run(tauri::generate_context!())
    {
        log::error!("❌ Erro ao executar aplicação Tauri: {:?}", e);
        panic!("Error while running tauri application: {:?}", e);
    }
}
