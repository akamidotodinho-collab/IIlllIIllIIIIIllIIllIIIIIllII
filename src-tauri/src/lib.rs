use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::Mutex;
use uuid::Uuid;
use chrono::Utc;

mod database_sqlite;
mod backup;
// mod ocr;  // Desabilitado - depende de tesseract
mod ocr_simple;
mod desktop;
mod date_extractor;

use database_sqlite::{Database, User};
use date_extractor::{DateExtractor, generate_folder_slug};
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
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(user) = authenticated_user.as_ref() {
        let logs = state.db.get_audit_logs(
            Some(&user.id),
            None,
            None,
            None,
            None,
            Some(10),
        ).map_err(|e| format!("Erro ao buscar atividades recentes: {:?}", e))?;
        
        let activities: Vec<ActivityResponse> = logs.into_iter().map(|log| {
            ActivityResponse {
                id: log.id,
                action: log.action,
                document: log.resource_name.unwrap_or_else(|| log.resource_type),
                timestamp: log.timestamp.format("%H:%M").to_string(),
                user: log.username,
            }
        }).collect();
        
        Ok(activities)
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

// Processar documento com OCR + IA - DESABILITADO (requer tesseract)
/*
#[tauri::command]
async fn process_document_ocr(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<OCRResult, String> {
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(user) = authenticated_user.as_ref() {
        let start_time = std::time::Instant::now();
        
        // Inicializar OCR processor se necessário
        let mut ocr_guard = state.ocr_processor.lock().await;
        if ocr_guard.is_none() {
            match ocr::create_ocr_processor() {
                Ok(processor) => {
                    *ocr_guard = Some(processor);
                    log::info!("✅ OCR Processor inicializado on-demand");
                }
                Err(e) => {
                    log::error!("❌ Erro ao inicializar OCR: {:?}", e);
                    return Err(format!("Erro ao inicializar OCR: {:?}", e));
                }
            }
        }
        
        if let Some(ocr_processor) = ocr_guard.as_mut() {
            // Determinar tipo do arquivo
            let path = std::path::Path::new(&file_path);
            let extension = path.extension()
                .and_then(|ext| ext.to_str())
                .map(|s| s.to_lowercase());
            
            let extracted_text = match extension.as_deref() {
                Some("pdf") => {
                    ocr_processor.extract_text_from_pdf(&file_path)
                        .map_err(|e| format!("Erro ao processar PDF: {:?}", e))?
                }
                Some("png") | Some("jpg") | Some("jpeg") | Some("tiff") | Some("bmp") => {
                    ocr_processor.extract_text_from_image(&file_path)
                        .map_err(|e| format!("Erro ao processar imagem: {:?}", e))?
                }
                _ => {
                    return Err("Tipo de arquivo não suportado. Use PDF, PNG, JPG, JPEG, TIFF ou BMP.".to_string());
                }
            };
            
            // Analisar documento com IA
            let metadata = ocr_processor.analyze_document(&extracted_text);
            let processing_time = start_time.elapsed().as_millis();
            
            // Log da operação na trilha de auditoria
            let file_name = path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("documento_desconhecido");
                
            let _ = log_audit_event(
                &state,
                &user.id,
                &user.username,
                "OCR_PROCESS",
                "DOCUMENT",
                Some(file_path.clone()),
                Some(file_name.to_string()),
                None,
                Some(serde_json::json!({
                    "document_type": format!("{:?}", metadata.document_type),
                    "confidence_score": metadata.confidence_score,
                    "processing_time_ms": processing_time,
                    "extracted_fields_count": metadata.extracted_fields.len()
                })),
                true,
            ).await;
            
            let result = OCRResult {
                extracted_text: metadata.text_content,
                document_type: format!("{:?}", metadata.document_type),
                extracted_fields: metadata.extracted_fields,
                confidence_score: metadata.confidence_score,
                processing_time_ms: processing_time,
            };
            
            log::info!("✅ OCR processamento concluído em {}ms", processing_time);
            Ok(result)
        } else {
            Err("OCR Processor não pôde ser inicializado".to_string())
        }
    } else {
        Err("Usuário não autenticado".to_string())
    }
}
*/  // Fim do bloco comentado process_document_ocr

// Obter tipos de documento suportados
#[tauri::command]
async fn get_supported_document_types() -> Result<Vec<String>, String> {
    Ok(ocr_simple::get_simple_supported_types())
}

// ================================
// COMANDO CREATE DOCUMENT COM DATE EXTRACTION
// ================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDocumentRequest {
    pub file_path: String,
    pub extracted_text: String,
    pub document_type: String,
    pub extracted_fields: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDocumentResponse {
    pub id: String,
    pub name: String,
    pub document_date: Option<String>,
    pub folder_slug: Option<String>,
    pub date_confidence: f32,
    pub date_source: String,
}

#[tauri::command]
async fn create_document(
    file_path: String,
    extracted_text: String,
    document_type: String,
    state: State<'_, AppState>,
) -> Result<CreateDocumentResponse, String> {
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(user) = authenticated_user.as_ref() {
        log::info!("📄 Criando documento: {}", file_path);
        
        // 1. EXTRAÇÃO AUTOMÁTICA DE DATA
        let date_extractor = DateExtractor::new();
        let path = std::path::Path::new(&file_path);
        let filename = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        
        let date_result = date_extractor.extract_date_auto(filename, &extracted_text);
        
        log::info!(
            "📅 Data extraída: {} (fonte: {:?}, confidence: {:.2})",
            date_result.value.format("%Y-%m-%d"),
            date_result.source,
            date_result.confidence
        );
        
        // 2. GERAR FOLDER SLUG
        let folder_slug = generate_folder_slug(&date_result.value);
        let document_date = date_result.value.format("%Y-%m-%d").to_string();
        
        // 3. OBTER METADADOS DO ARQUIVO
        let file_metadata = std::fs::metadata(&file_path)
            .map_err(|e| format!("Erro ao ler metadados do arquivo: {:?}", e))?;
        
        let file_size = file_metadata.len() as i64;
        let file_type = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        // 4. CRIAR DOCUMENTO NO BANCO
        let doc_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let document = database_sqlite::Document {
            id: doc_id.clone(),
            user_id: user.id.clone(),
            name: filename.to_string(),
            file_path: file_path.clone(),
            file_type: file_type.clone(),
            file_size,
            created_at: now,
            updated_at: now,
            tags: vec![],
            document_date: Some(document_date.clone()),
            folder_slug: Some(folder_slug.clone()),
        };
        
        state.db.create_document(&document)
            .map_err(|e| format!("Erro ao criar documento no banco: {:?}", e))?;
        
        // 5. LOG NA TRILHA DE AUDITORIA
        let _ = log_audit_event(
            &state,
            &user.id,
            &user.username,
            "DOCUMENT_CREATE",
            "DOCUMENT",
            Some(doc_id.clone()),
            Some(filename.to_string()),
            None,
            Some(serde_json::json!({
                "file_path": file_path,
                "document_type": document_type,
                "document_date": document_date,
                "folder_slug": folder_slug,
                "date_source": format!("{:?}", date_result.source),
                "date_confidence": date_result.confidence,
                "file_size": file_size,
            })),
            true,
        ).await;
        
        log::info!("✅ Documento criado: {} (pasta: {})", doc_id, folder_slug);
        
        Ok(CreateDocumentResponse {
            id: doc_id,
            name: filename.to_string(),
            document_date: Some(document_date),
            folder_slug: Some(folder_slug),
            date_confidence: date_result.confidence,
            date_source: format!("{:?}", date_result.source),
        })
    } else {
        Err("Usuário não autenticado".to_string())
    }
}

// ================================
// COMANDOS DE ORGANIZAÇÃO POR PASTAS
// ================================

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderInfo {
    pub folder_slug: String,
    pub document_count: i64,
}

#[tauri::command]
async fn get_available_folders(
    state: State<'_, AppState>,
) -> Result<Vec<FolderInfo>, String> {
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(user) = authenticated_user.as_ref() {
        let folders = state.db.get_available_folders(&user.id)
            .map_err(|e| format!("Erro ao buscar pastas: {:?}", e))?;
        
        let response: Vec<FolderInfo> = folders.into_iter().map(|(slug, count)| {
            FolderInfo {
                folder_slug: slug,
                document_count: count,
            }
        }).collect();
        
        log::debug!("📂 Retornando {} pastas virtuais", response.len());
        Ok(response)
    } else {
        Err("Usuário não autenticado".to_string())
    }
}

#[tauri::command]
async fn get_documents_by_folder(
    folder_slug: String,
    state: State<'_, AppState>,
) -> Result<Vec<DocumentResponse>, String> {
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(user) = authenticated_user.as_ref() {
        let documents = state.db.get_documents_by_folder(&user.id, &folder_slug)
            .map_err(|e| format!("Erro ao buscar documentos da pasta: {:?}", e))?;
        
        let response: Vec<DocumentResponse> = documents.into_iter().map(|doc| {
            DocumentResponse {
                id: doc.id,
                name: doc.name,
                size: doc.file_size,
                file_type: doc.file_type,
                upload_date: doc.document_date.unwrap_or_else(|| doc.created_at.format("%Y-%m-%d").to_string()),
                is_active: true,
                category: doc.folder_slug.unwrap_or_else(|| "Sem pasta".to_string()),
                preview_available: false,
            }
        }).collect();
        
        log::debug!("📁 Retornando {} documentos da pasta {}", response.len(), folder_slug);
        Ok(response)
    } else {
        Err("Usuário não autenticado".to_string())
    }
}

#[tauri::command]
async fn get_documents_by_date_range(
    start_date: String,
    end_date: String,
    state: State<'_, AppState>,
) -> Result<Vec<DocumentResponse>, String> {
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(user) = authenticated_user.as_ref() {
        let documents = state.db.get_documents_by_date_range(&user.id, &start_date, &end_date)
            .map_err(|e| format!("Erro ao buscar documentos por data: {:?}", e))?;
        
        let response: Vec<DocumentResponse> = documents.into_iter().map(|doc| {
            DocumentResponse {
                id: doc.id,
                name: doc.name,
                size: doc.file_size,
                file_type: doc.file_type,
                upload_date: doc.document_date.unwrap_or_else(|| doc.created_at.format("%Y-%m-%d").to_string()),
                is_active: true,
                category: doc.folder_slug.unwrap_or_else(|| "Sem pasta".to_string()),
                preview_available: false,
            }
        }).collect();
        
        log::debug!("📅 Retornando {} documentos entre {} e {}", response.len(), start_date, end_date);
        Ok(response)
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
    pub relevance_score: f32,
    pub matched_content: String,
    pub created_at: String,
}

// Buscar documentos por texto
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
        
        if query.trim().is_empty() {
            return Err("Query de busca não pode estar vazia".to_string());
        }
        
        // Obter estatísticas
        let (total_docs, indexed_docs) = state.db.get_search_stats(&user.id)
            .map_err(|e| format!("Erro ao obter estatísticas: {:?}", e))?;
        
        // Executar busca (FTS5 ou fallback)
        let results = if use_fts.unwrap_or(true) {
            // Tentar busca FTS5 primeiro
            match state.db.search_documents(&user.id, &query, limit) {
                Ok(results) => results,
                Err(e) => {
                    log::warn!("FTS5 falhou, usando busca simples: {:?}", e);
                    state.db.simple_search_documents(&user.id, &query, limit)
                        .map_err(|e| format!("Erro na busca: {:?}", e))?
                }
            }
        } else {
            // Busca simples
            state.db.simple_search_documents(&user.id, &query, limit)
                .map_err(|e| format!("Erro na busca simples: {:?}", e))?
        };
        
        let search_time = start_time.elapsed().as_millis();
        
        // Log da busca na trilha de auditoria
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
                "results_count": results.len(),
                "search_time_ms": search_time,
                "fts_enabled": use_fts.unwrap_or(true)
            })),
            true,
        ).await;
        
        // Converter para response format
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
        log::info!("🔍 Busca '{}' concluída em {}ms - {} resultados", 
                  query, search_time, total_found);
        
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

// Indexar documento após processamento OCR
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
        state.db.index_document_content(
            &document_id,
            &extracted_text,
            &document_type,
            &extracted_fields,
        ).map_err(|e| format!("Erro ao indexar documento: {:?}", e))?;
        
        // Log da indexação
        let doc_id_clone = document_id.clone();
        let _ = log_audit_event(
            &state,
            &user.id,
            &user.username,
            "INDEX",
            "DOCUMENT",
            Some(document_id),
            None,
            None,
            Some(serde_json::json!({
                "document_type": document_type,
                "text_length": extracted_text.len(),
                "fields_count": extracted_fields.as_object().map(|o| o.len()).unwrap_or(0)
            })),
            true,
        ).await;
        
        log::info!("📝 Documento {} indexado com sucesso", doc_id_clone);
        Ok(true)
    } else {
        Err("Usuário não autenticado".to_string())
    }
}

// Obter estatísticas de busca
#[tauri::command]
async fn get_search_statistics(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(user) = authenticated_user.as_ref() {
        let (total_docs, indexed_docs) = state.db.get_search_stats(&user.id)
            .map_err(|e| format!("Erro ao obter estatísticas: {:?}", e))?;
        
        let indexing_percentage = if total_docs > 0 {
            (indexed_docs as f64 / total_docs as f64 * 100.0) as u32
        } else {
            0
        };
        
        Ok(serde_json::json!({
            "total_documents": total_docs,
            "indexed_documents": indexed_docs,
            "indexing_percentage": indexing_percentage,
            "fts5_available": true
        }))
    } else {
        Err("Usuário não autenticado".to_string())
    }
}

// ================================
// COMANDO DOWNLOAD NATIVO
// ================================

// Download de documento com save-as dialog nativo
#[tauri::command]
async fn download_document(
    document_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(user) = authenticated_user.as_ref() {
        // Buscar documento no banco
        let documents = state.db.get_documents_by_user(&user.id)
            .map_err(|e| format!("Erro ao buscar documento: {:?}", e))?;
            
        let document = documents.into_iter()
            .find(|doc| doc.id == document_id)
            .ok_or_else(|| "Documento não encontrado".to_string())?;
            
        // Dialog save-as nativo (será implementado via plugin-dialog no frontend)
        log::info!("📥 Download solicitado: {} ({})", document.name, document.file_type);
        
        // Log da operação na trilha de auditoria
        let _ = log_audit_event(
            &state,
            &user.id,
            &user.username,
            "DOWNLOAD",
            "DOCUMENT",
            Some(document.id.clone()),
            Some(document.name.clone()),
            None,
            Some(serde_json::json!({
                "file_name": document.name,
                "file_size": document.file_size,
                "file_type": document.file_type
            })),
            true,
        ).await;
        
        Ok(true)
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
            create_document,
            get_available_folders,
            get_documents_by_folder,
            get_documents_by_date_range,
            search_documents,
            index_document_for_search,
            get_search_statistics,
            backup::verify_backup_file,
            backup::list_available_backups,
            download_document,
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
