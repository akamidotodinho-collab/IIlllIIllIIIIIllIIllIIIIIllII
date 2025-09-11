use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

mod database_sqlite;
mod backup;

use database_sqlite::{Database, User, AuditLog};
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

// Estruturas para trilha de auditoria
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLogResponse {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub resource_name: Option<String>,
    pub ip_address: Option<String>,
    pub file_hash: Option<String>,
    pub current_hash: String,
    pub metadata: String,
    pub timestamp: String,
    pub is_success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditChainStatus {
    pub is_valid: bool,
    pub total_logs: usize,
    pub first_log_date: Option<String>,
    pub last_log_date: Option<String>,
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
                
                // REGISTRAR LOGIN SUCESSO NA TRILHA DE AUDITORIA
                let _ = log_audit_event(
                    &state,
                    &user.id,
                    &user.username,
                    "LOGIN",
                    "SYSTEM",
                    None,
                    None,
                    None,
                    Some(serde_json::json!({"ip_address": "local", "success": true})),
                    true,
                ).await;
                
                Ok(LoginResponse {
                    success: true,
                    user: Some(user.username),
                    error: None,
                })
            } else {
                // REGISTRAR LOGIN FALHA NA TRILHA DE AUDITORIA
                let _ = log_audit_event(
                    &state,
                    &user.id,
                    &user.username,
                    "LOGIN_FAILED",
                    "SYSTEM",
                    None,
                    None,
                    None,
                    Some(serde_json::json!({"ip_address": "local", "reason": "invalid_password"})),
                    false,
                ).await;
                
                Ok(LoginResponse {
                    success: false,
                    user: None,
                    error: Some("Senha incorreta".to_string()),
                })
            }
        }
        Ok(None) => {
            // REGISTRAR TENTATIVA DE LOGIN COM USUÁRIO INEXISTENTE NA TRILHA DE AUDITORIA
            let _ = state.db.create_audit_log(
                "UNKNOWN_USER",
                &request.username,
                "LOGIN_FAILED",
                "SYSTEM",
                None,
                None,
                None,
                None,
                None,
                Some(serde_json::json!({
                    "ip_address": "local", 
                    "reason": "user_not_found",
                    "attempted_username": request.username
                })),
                false,
            );
            
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

// ================================
// COMANDOS DE TRILHA DE AUDITORIA LEGAL
// ================================

// Buscar logs de auditoria
#[tauri::command]
async fn get_audit_logs(
    action: Option<String>,
    resource_type: Option<String>,
    days_back: Option<u32>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<AuditLogResponse>, String> {
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(user) = authenticated_user.as_ref() {
        // Calcular data de início se days_back foi fornecido
        let start_date = days_back.map(|days| {
            chrono::Utc::now() - chrono::Duration::days(days as i64)
        });
        
        let logs = state.db.get_audit_logs(
            Some(&user.id),
            action.as_deref(),
            resource_type.as_deref(),
            start_date,
            None,
            limit,
        ).map_err(|e| format!("Erro ao buscar logs de auditoria: {:?}", e))?;
        
        let response: Vec<AuditLogResponse> = logs.into_iter().map(|log| {
            AuditLogResponse {
                id: log.id,
                user_id: log.user_id,
                username: log.username,
                action: log.action,
                resource_type: log.resource_type,
                resource_id: log.resource_id,
                resource_name: log.resource_name,
                ip_address: log.ip_address,
                file_hash: log.file_hash,
                current_hash: log.current_hash,
                metadata: log.metadata,
                timestamp: log.timestamp.format("%d/%m/%Y %H:%M:%S").to_string(),
                is_success: log.is_success,
            }
        }).collect();
        
        Ok(response)
    } else {
        Err("Usuário não autenticado".to_string())
    }
}

// Verificar integridade da cadeia de auditoria
#[tauri::command]
async fn verify_audit_chain(
    state: State<'_, AppState>,
) -> Result<AuditChainStatus, String> {
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(_user) = authenticated_user.as_ref() {
        let is_valid = state.db.verify_audit_chain()
            .map_err(|e| format!("Erro ao verificar cadeia de auditoria: {:?}", e))?;
        
        // Buscar estatísticas da cadeia usando nova função otimizada
        let (total_logs, first_log_date, last_log_date) = state.db.get_audit_chain_stats()
            .map_err(|e| format!("Erro ao buscar estatísticas: {:?}", e))?;
        
        Ok(AuditChainStatus {
            is_valid,
            total_logs,
            first_log_date,
            last_log_date,
        })
    } else {
        Err("Usuário não autenticado".to_string())
    }
}

// Função para registrar automaticamente logs de auditoria (uso interno)
pub async fn log_audit_event(
    state: &AppState,
    user_id: &str,
    username: &str,
    action: &str,
    resource_type: &str,
    resource_id: Option<String>,
    resource_name: Option<String>,
    file_hash: Option<String>,
    metadata: Option<serde_json::Value>,
    is_success: bool,
) -> Result<(), String> {
    state.db.create_audit_log(
        user_id,
        username,
        action,
        resource_type,
        resource_id,
        resource_name,
        None, // ip_address - TODO: implementar detecção de IP
        None, // user_agent - TODO: implementar detecção de User-Agent
        file_hash,
        metadata,
        is_success,
    ).map_err(|e| format!("Erro ao criar log de auditoria: {:?}", e))?;
    
    Ok(())
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
    log::info!("🚀 Iniciando ARKIVE Desktop...");
    
    log::info!("🔧 Inicializando AppState...");
    let app_state = match AppState::new() {
        Ok(state) => {
            log::info!("✅ AppState inicializado com sucesso");
            state
        },
        Err(e) => {
            log::error!("❌ Falha ao inicializar AppState: {:?}", e);
            eprintln!("ERRO ARKIVE: Falha ao inicializar banco de dados: {:?}", e);
            std::process::exit(1);
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
            get_audit_logs,
            verify_audit_chain,
            backup::verify_backup_file,
            backup::list_available_backups,
            test_audit_security,
        ])
        .run(tauri::generate_context!())
    {
        log::error!("❌ Erro ao executar aplicação Tauri: {:?}", e);
        eprintln!("ERRO ARKIVE: Falha ao iniciar aplicação: {:?}", e);
        std::process::exit(1);
    }
}
