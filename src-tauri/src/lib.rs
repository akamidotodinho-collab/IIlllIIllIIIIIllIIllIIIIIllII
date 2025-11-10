use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use uuid::Uuid;
use chrono::Utc;

mod database_sqlite;
mod backup;
// mod ocr;  // Desabilitado - depende de tesseract
mod ocr_simple;
mod desktop;

use database_sqlite::{Database, User};
// use ocr::{OCRProcessor, ExtractedMetadata, DocumentType};  // Desabilitado
use ocr_simple::{SimpleOCRResult, create_simple_ocr_processor};
use std::path::PathBuf;

// Estado da aplicação
pub struct AppState {
    pub db: Arc<Database>,
    pub authenticated_user: Arc<Mutex<Option<User>>>,
    // pub ocr_processor: Arc<Mutex<Option<OCRProcessor>>>,  // Desabilitado
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
        
        // OCR processor desabilitado - usa SimpleOCR apenas
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
    username: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("🔐 Tentativa de login: {}", username);
    let user_result = state.db.get_user_by_username(&username);
    
    match user_result {
        Ok(Some(user)) => {
            if bcrypt::verify(&password, &user.password_hash).unwrap_or(false) {
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
                
                log::info!("✅ Login bem-sucedido: {}", username);
                // Retornar User completo como JSON
                let user_json = serde_json::json!({
                    "id": user.id,
                    "username": user.username,
                    "created_at": user.created_at.to_rfc3339()
                });
                Ok(user_json.to_string())
            } else {
                log::warn!("❌ Senha incorreta: {}", username);
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
                
                Err("Senha incorreta".to_string())
            }
        }
        Ok(None) => {
            log::warn!("❌ Usuário não encontrado: {}", username);
            // REGISTRAR TENTATIVA DE LOGIN COM USUÁRIO INEXISTENTE NA TRILHA DE AUDITORIA
            let _ = state.db.create_audit_log(
                "UNKNOWN_USER",
                &username,
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
                    "attempted_username": username
                })),
                false,
            );
            
            Err("Usuário não encontrado".to_string())
        }
        Err(e) => {
            log::error!("❌ Erro de banco: {:?}", e);
            Err(format!("Erro de banco: {:?}", e))
        }
    }
}

#[tauri::command]
async fn register(
    username: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("📝 Tentativa de registro: {}", username);
    
    if password.len() < 6 {
        log::warn!("❌ Senha muito curta");
        return Err("Senha deve ter pelo menos 6 caracteres".to_string());
    }

    // Criar usuário
    let password_hash = match bcrypt::hash(&password, 12) {
        Ok(hash) => hash,
        Err(e) => {
            log::error!("❌ Erro ao gerar hash: {:?}", e);
            return Err(format!("Erro ao criptografar senha: {:?}", e));
        }
    };

    // Criar objeto User completo
    let user = User {
        id: Uuid::new_v4().to_string(),
        username: username.clone(),
        email: format!("{}@local", username),
        password_hash,
        created_at: Utc::now(),
        last_login: None,
    };

    match state.db.create_user(&user) {
        Ok(_) => {
            let mut authenticated_user = state.authenticated_user.lock().await;
            *authenticated_user = Some(user.clone());
            
            // REGISTRAR REGISTRO NA TRILHA DE AUDITORIA
            let _ = log_audit_event(
                &state,
                &user.id,
                &user.username,
                "REGISTER",
                "SYSTEM",
                None,
                None,
                None,
                Some(serde_json::json!({"ip_address": "local"})),
                true,
            ).await;
            
            log::info!("✅ Usuário registrado: {}", username);
            // Retornar User completo como JSON
            let user_json = serde_json::json!({
                "id": user.id,
                "username": user.username,
                "created_at": user.created_at.to_rfc3339()
            });
            Ok(user_json.to_string())
        }
        Err(e) => {
            log::error!("❌ Erro ao criar usuário: {:?}", e);
            Err(format!("Erro ao criar usuário: {:?}. Usuário pode já existir.", e))
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

// ================================
// COMANDOS OCR + IA OFFLINE
// ================================

#[derive(Debug, Serialize, Deserialize)]
pub struct OCRResult {
    pub extracted_text: String,
    pub document_type: String,
    pub extracted_fields: std::collections::HashMap<String, String>,
    pub confidence_score: f32,
    pub processing_time_ms: u128,
}

// Novo comando OCR simplificado e confiável
#[tauri::command]
async fn process_document_simple_ocr(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<SimpleOCRResult, String> {
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(user) = authenticated_user.as_ref() {
        log::info!("🔍 Iniciando OCR simplificado para: {}", file_path);
        
        let processor = create_simple_ocr_processor()
            .map_err(|e| format!("Erro ao criar OCR processor: {:?}", e))?;
        
        let path = std::path::Path::new(&file_path);
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase());
        
        let result = match extension.as_deref() {
            Some("pdf") => {
                processor.process_pdf(&file_path).await
                    .map_err(|e| format!("Erro ao processar PDF: {:?}", e))?
            }
            Some("png") | Some("jpg") | Some("jpeg") | Some("tiff") | Some("bmp") => {
                processor.process_image(&file_path).await
                    .map_err(|e| format!("Erro ao processar imagem: {:?}", e))?
            }
            _ => {
                return Err("Tipo de arquivo não suportado. Use PDF, PNG, JPG, JPEG, TIFF ou BMP.".to_string());
            }
        };
        
        // Log da operação
        let file_name = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("documento_desconhecido");
        
        let _ = log_audit_event(
            &state,
            &user.id,
            &user.username,
            "OCR_SIMPLE",
            "DOCUMENT",
            Some(file_name.to_string()),
            Some(file_name.to_string()),
            None,
            Some(serde_json::json!({
                "file_path": file_path,
                "document_type": result.document_type,
                "confidence_score": result.confidence_score,
                "processing_time_ms": result.processing_time_ms,
                "method": result.processing_method
            })),
            result.error_message.is_none(),
        ).await;
        
        Ok(result)
    } else {
        Err("Usuário não autenticado".to_string())
    }
}

// Obter tipos de documento suportados
#[tauri::command]
async fn get_supported_document_types() -> Result<Vec<String>, String> {
    Ok(ocr_simple::get_simple_supported_types())
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

// ================================
// COMANDOS DE BUSCA FULL-TEXT FTS5
// ================================

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResultResponse>,
    pub total_found: usize,
    pub search_time_ms: u128,
    pub indexed_docs: i64,
    pub total_docs: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResultResponse {
    pub document_id: String,
    pub document_name: String,
    pub document_type: String,
    pub file_path: String,
    pub relevance_score: f64,
    pub matched_content: String,
    pub created_at: String,
}

#[tauri::command]
async fn search_documents(
    query: String,
    limit: Option<usize>,
    use_fts: Option<bool>,
    state: State<'_, AppState>,
) -> Result<SearchResponse, String> {
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(user) = authenticated_user.as_ref() {
        let start_time = std::time::Instant::now();
        
        // Usar FTS5 por padrão
        let use_fts5 = use_fts.unwrap_or(true);
        let search_limit = limit.unwrap_or(50);
        
        log::info!("🔍 Buscando documentos: query='{}', fts={}, limit={}", query, use_fts5, search_limit);
        
        let results = if use_fts5 {
            state.db.search_documents_fts(&user.id, &query, search_limit)
                .map_err(|e| format!("Erro na busca FTS5: {:?}", e))?
        } else {
            state.db.search_documents_simple(&user.id, &query, search_limit)
                .map_err(|e| format!("Erro na busca simples: {:?}", e))?
        };
        
        let search_time = start_time.elapsed().as_millis();
        
        // Buscar estatísticas
        let (indexed_docs, total_docs) = state.db.get_search_stats(&user.id)
            .map_err(|e| format!("Erro ao buscar estatísticas: {:?}", e))?;
        
        let response_results: Vec<SearchResultResponse> = results.into_iter().map(|r| {
            SearchResultResponse {
                document_id: r.document_id,
                document_name: r.document_name,
                document_type: r.document_type,
                file_path: r.file_path,
                relevance_score: r.relevance_score,
                matched_content: r.matched_content,
                created_at: r.created_at.format("%d/%m/%Y %H:%M").to_string(),
            }
        }).collect();
        
        let total_found = response_results.len();
        
        log::info!("✅ Busca concluída: {} resultados em {}ms", total_found, search_time);
        
        // Log da operação
        let _ = log_audit_event(
            &state,
            &user.id,
            &user.username,
            "SEARCH",
            "DOCUMENT",
            None,
            None,
            None,
            Some(serde_json::json!({
                "query": query,
                "results_found": total_found,
                "search_time_ms": search_time,
                "use_fts5": use_fts5
            })),
            true,
        ).await;
        
        Ok(SearchResponse {
            results: response_results,
            total_found,
            search_time_ms: search_time,
            indexed_docs,
            total_docs,
        })
    } else {
        Err("Usuário não autenticado".to_string())
    }
}

#[tauri::command]
async fn index_document_for_search(
    document_id: String,
    extracted_text: String,
    document_type: String,
    extracted_fields: serde_json::Value,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(user) = authenticated_user.as_ref() {
        log::info!("📇 Indexando documento {} para busca FTS5", document_id);
        
        state.db.index_document_content(
            &document_id,
            &extracted_text,
            &document_type,
            extracted_fields,
        ).map_err(|e| format!("Erro ao indexar documento: {:?}", e))?;
        
        // Log da operação
        let _ = log_audit_event(
            &state,
            &user.id,
            &user.username,
            "INDEX_DOCUMENT",
            "SEARCH",
            Some(document_id.clone()),
            None,
            None,
            Some(serde_json::json!({
                "document_id": document_id,
                "document_type": document_type,
                "text_length": extracted_text.len()
            })),
            true,
        ).await;
        
        log::info!("✅ Documento indexado com sucesso");
        Ok(true)
    } else {
        Err("Usuário não autenticado".to_string())
    }
}

#[tauri::command]
async fn get_search_statistics(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(user) = authenticated_user.as_ref() {
        let (indexed_docs, total_docs) = state.db.get_search_stats(&user.id)
            .map_err(|e| format!("Erro ao buscar estatísticas: {:?}", e))?;
        
        Ok(serde_json::json!({
            "indexed_documents": indexed_docs,
            "total_documents": total_docs,
            "indexing_coverage": if total_docs > 0 {
                (indexed_docs as f64 / total_docs as f64) * 100.0
            } else {
                0.0
            }
        }))
    } else {
        Err("Usuário não autenticado".to_string())
    }
}

// ================================
// COMANDOS DE DOCUMENTOS (CRUD)
// ================================

#[tauri::command]
async fn download_document(
    document_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let authenticated_user = state.authenticated_user.lock().await;
        if let Some(user) = authenticated_user.as_ref() {
        // Buscar informações do documento
        let document = state.db.get_document_by_id(&document_id)
            .map_err(|e| format!("Erro ao buscar documento: {:?}", e))?;
        
        if let Some(doc) = document {
            // Verificar se o documento pertence ao usuário
            if doc.user_id != user.id {
                return Err("Acesso negado: documento não pertence ao usuário".to_string());
            }
            
            // Usar dialog nativo para salvar arquivo
            use tauri::Manager;
            let app_handle = state.db.clone(); // Placeholder - precisa do app_handle real
            
            // Abrir dialog "Save As" nativo
            match desktop::save_backup_dialog().await {
                Ok(save_path) => {
                    if let Some(dest_path) = save_path {
                        // Copiar arquivo para destino escolhido
                        std::fs::copy(&doc.file_path, &dest_path)
                            .map_err(|e| format!("Erro ao copiar arquivo: {:?}", e))?;
                        
                        // Log da operação
                        let _ = log_audit_event(
                            &state,
                            &user.id,
                            &user.username,
                            "DOWNLOAD",
                            "DOCUMENT",
                            Some(document_id.clone()),
                            Some(doc.name.clone()),
                            None,
                            Some(serde_json::json!({
                                "document_id": document_id,
                                "destination": dest_path
                            })),
                            true,
                        ).await;
                        
                        log::info!("✅ Download concluído: {}", doc.name);
                        Ok(true)
                    } else {
                        Ok(false) // Usuário cancelou
                    }
                }
                Err(e) => {
                    Err(format!("Erro ao abrir dialog: {:?}", e))
                }
            }
        } else {
            Err("Documento não encontrado".to_string())
        }
    } else {
        Err("Usuário não autenticado".to_string())
    }
}

// Helper function para formatar tamanho
fn format_size(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

// ================================
// COMANDOS PARA CRIAR DOCUMENTO
// ================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDocumentRequest {
    pub file_path: String,
    pub ocr_result: serde_json::Value,
}

#[tauri::command]
async fn create_document(
    file_path: String,
    ocr_result: serde_json::Value,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(user) = authenticated_user.as_ref() {
        log::info!("📄 Criando documento: {}", file_path);
        
        let path = std::path::Path::new(&file_path);
        let file_name = path.file_name()
            .and_then(|name| name.to_str())
            .ok_or("Nome de arquivo inválido")?;
        
        let file_type = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_else(|| "unknown".to_string());
        
        let file_size = std::fs::metadata(&file_path)
            .map(|m| m.len() as i64)
            .unwrap_or(0);
        
        // Criar documento no banco
        let doc_id = state.db.create_document(
            &user.id,
            file_name,
            &file_path,
            &file_type,
            file_size,
        ).map_err(|e| format!("Erro ao criar documento: {:?}", e))?;
        
        // Log da operação
        let _ = log_audit_event(
            &state,
            &user.id,
            &user.username,
            "CREATE_DOCUMENT",
            "DOCUMENT",
            Some(doc_id.clone()),
            Some(file_name.to_string()),
            None,
            Some(serde_json::json!({
                "document_id": doc_id,
                "file_name": file_name,
                "file_size": file_size,
                "ocr_result": ocr_result
            })),
            true,
        ).await;
        
        log::info!("✅ Documento criado: {}", doc_id);
        
        // Retornar documento como JSON
        let doc_json = serde_json::json!({
            "id": doc_id,
            "name": file_name,
            "file_size": file_size,
            "file_type": file_type,
            "created_at": chrono::Utc::now().to_rfc3339()
        });
        
        Ok(doc_json.to_string())
    } else {
        Err("Usuário não autenticado".to_string())
    }
}

#[tauri::command]
async fn delete_document(
    document_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(user) = authenticated_user.as_ref() {
        log::info!("🗑️ Deletando documento: {}", document_id);
        
        // Buscar documento para verificar propriedade
        let document = state.db.get_document_by_id(&document_id)
            .map_err(|e| format!("Erro ao buscar documento: {:?}", e))?;
        
        if let Some(doc) = document {
            if doc.user_id != user.id {
                return Err("Acesso negado: documento não pertence ao usuário".to_string());
            }
            
            // Deletar do banco
            state.db.delete_document(&document_id)
                .map_err(|e| format!("Erro ao deletar documento: {:?}", e))?;
            
            // Tentar deletar arquivo físico (não crítico se falhar)
            let _ = std::fs::remove_file(&doc.file_path);
            
            // Log da operação
            let _ = log_audit_event(
                &state,
                &user.id,
                &user.username,
                "DELETE_DOCUMENT",
                "DOCUMENT",
                Some(document_id.clone()),
                Some(doc.name.clone()),
                None,
                Some(serde_json::json!({
                    "document_id": document_id,
                    "file_name": doc.name
                })),
                true,
            ).await;
            
            log::info!("✅ Documento deletado: {}", document_id);
            Ok(true)
        } else {
            Err("Documento não encontrado".to_string())
        }
    } else {
        Err("Usuário não autenticado".to_string())
    }
}

// ================================
// ENTRY POINT - MAIN
// ================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Inicializar AppState antes de tudo
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
    
    // Configurar builder do Tauri com tratamento robusto de erro
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build())
        .setup(|app| {
            // Inicializar AppState na setup do Tauri para melhor tratamento de erro
            log::info!("🔧 Configurando aplicação...");
            
            // Criar diretório de logs para Windows
            if let Ok(app_dir) = app.path().app_data_dir() {
                let log_dir = app_dir.join("logs");
                if let Err(e) = std::fs::create_dir_all(&log_dir) {
                    log::warn!("Não foi possível criar diretório de logs: {:?}", e);
                }
            }
            
            log::info!("✅ Setup concluído com sucesso");
            Ok(())
        })
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
            // process_document_ocr,  // Desabilitado - requer tesseract
            process_document_simple_ocr,
            get_supported_document_types,
            search_documents,
            index_document_for_search,
            get_search_statistics,
            backup::verify_backup_file,
            backup::list_available_backups,
            download_document,
            create_document,
            delete_document,
            desktop::open_file_dialog,
            desktop::save_backup_dialog,
            desktop::open_in_explorer,
        ]);

    // Executar aplicação com tratamento de erro melhorado
    if let Err(e) = builder.run(tauri::generate_context!()) {
        log::error!("❌ Erro ao executar aplicação Tauri: {:?}", e);
        
        // Tentar mostrar erro para o usuário no Windows
        eprintln!("ERRO ARKIVE: {}", e);
        
        // Se for erro crítico de inicialização, tentar mostrar dialog
        if let Ok(_) = std::env::var("DISPLAY") {
            // Linux/macOS - mostrar no terminal
            eprintln!("Por favor, verifique as dependências do sistema.");
        } else {
            // Windows - tentar criar arquivo de erro visível
            if let Ok(mut error_file) = std::fs::File::create("arkive_error.txt") {
                use std::io::Write;
                let _ = writeln!(error_file, "ERRO ARKIVE: {}", e);
                let _ = writeln!(error_file, "\nPossíveis soluções:");
                let _ = writeln!(error_file, "1. Instalar WebView2: winget install Microsoft.EdgeWebView2Runtime");
                let _ = writeln!(error_file, "2. Instalar Visual C++: winget install Microsoft.VCRedist.2015+");
                let _ = writeln!(error_file, "3. Executar como administrador");
            }
        }
        
        std::process::exit(1);
    }
}
