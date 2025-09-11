use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result as SqliteResult, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use std::time::Duration;
use std::thread;
use sha2::{Sha256, Digest};
use std::fmt::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub file_path: String,
    pub file_type: String,
    pub file_size: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub user_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub details: String,
    pub created_at: DateTime<Utc>,
}

// SISTEMA DE BUSCA FULL-TEXT COM FTS5
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContent {
    pub document_id: String,
    pub extracted_text: String,
    pub document_type: String,
    pub extracted_fields: String, // JSON
    pub indexed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document_id: String,
    pub document_name: String,
    pub document_type: String,
    pub file_path: String,
    pub relevance_score: f32,
    pub matched_content: String,
    pub created_at: DateTime<Utc>,
}

// TRILHA DE AUDITORIA LEGAL - IMUTÁVEL E CRIPTOGRAFICAMENTE SEGURA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub sequence_id: i64,              // Número sequencial monotônico (PRIMARY KEY)
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub action: String,          // LOGIN, UPLOAD, DOWNLOAD, VIEW, DELETE, MODIFY
    pub resource_type: String,   // DOCUMENT, USER, SYSTEM
    pub resource_id: Option<String>,
    pub resource_name: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub file_hash: Option<String>,     // SHA-256 do arquivo
    pub previous_hash: String,         // Hash do log anterior (blockchain-like)
    pub current_hash: String,          // SHA-256 deste log (imutabilidade)
    pub metadata: String,              // JSON com detalhes extras
    pub timestamp: DateTime<Utc>,      // Timestamp preciso
    pub is_success: bool,              // Se a ação foi bem-sucedida
}

pub struct Database {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl Database {
    pub fn new(db_path: PathBuf) -> SqliteResult<Self> {
        let conn = Connection::open(&db_path)?;
        
        // CONFIGURAÇÕES CRÍTICAS DE PERFORMANCE E CONCORRÊNCIA
        conn.execute_batch(r#"
            -- WAL mode para melhor concorrência
            PRAGMA journal_mode = WAL;
            
            -- Sincronização otimizada
            PRAGMA synchronous = NORMAL;
            
            -- Cache aumentado (10MB)
            PRAGMA cache_size = 10000;
            
            -- Timeout para locks (30 segundos)
            PRAGMA busy_timeout = 30000;
            
            -- Auto checkpoint otimizado
            PRAGMA wal_autocheckpoint = 1000;
            
            -- Memory mapping (melhor I/O)
            PRAGMA mmap_size = 268435456;
            
            -- Temp store na memória
            PRAGMA temp_store = memory;
        "#)?;
        
        let database = Database {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        };
        
        database.create_tables()?;
        Ok(database)
    }
    
    fn create_tables(&self) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        
        // Tabela de usuários
        conn.execute(r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_login TEXT
            )
        "#, [])?;
        
        // Tabela de documentos
        conn.execute(r#"
            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                name TEXT NOT NULL,
                file_path TEXT NOT NULL,
                file_type TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                tags TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users (id)
            )
        "#, [])?;
        
        // Tabela de atividades
        conn.execute(r#"
            CREATE TABLE IF NOT EXISTS activities (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                action TEXT NOT NULL,
                resource_type TEXT NOT NULL,
                resource_id TEXT NOT NULL,
                details TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users (id)
            )
        "#, [])?;
        
        // TABELA DE AUDITORIA LEGAL - IMUTÁVEL E CRIPTOGRAFICAMENTE SEGURA
        // APPEND-ONLY COM PROTEÇÃO CONTRA ADULTERAÇÃO
        conn.execute(r#"
            CREATE TABLE IF NOT EXISTS audit_logs (
                sequence_id INTEGER PRIMARY KEY AUTOINCREMENT,
                id TEXT UNIQUE NOT NULL,
                user_id TEXT NOT NULL,
                username TEXT NOT NULL,
                action TEXT NOT NULL,
                resource_type TEXT NOT NULL,
                resource_id TEXT,
                resource_name TEXT,
                ip_address TEXT,
                user_agent TEXT,
                file_hash TEXT,
                previous_hash TEXT NOT NULL,
                current_hash TEXT NOT NULL,
                metadata TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                is_success BOOLEAN NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users (id)
            )
        "#, [])?;
        
        // TRIGGERS CRÍTICOS DE SEGURANÇA - IMPEDEM ADULTERAÇÃO DA TRILHA DE AUDITORIA
        // Bloquear UPDATE nos logs de auditoria (IMUTABILIDADE)
        conn.execute(r#"
            CREATE TRIGGER IF NOT EXISTS prevent_audit_log_update
            BEFORE UPDATE ON audit_logs
            BEGIN
                SELECT RAISE(ABORT, 'TRILHA DE AUDITORIA IMUTÁVEL: UPDATE proibido por questões legais e de segurança');
            END
        "#, [])?;
        
        // Bloquear DELETE nos logs de auditoria (APPEND-ONLY)
        conn.execute(r#"
            CREATE TRIGGER IF NOT EXISTS prevent_audit_log_delete
            BEFORE DELETE ON audit_logs
            BEGIN
                SELECT RAISE(ABORT, 'TRILHA DE AUDITORIA IMUTÁVEL: DELETE proibido por questões legais e de segurança');
            END
        "#, [])?;
        
        // ÍNDICES PARA PERFORMANCE
        conn.execute("CREATE INDEX IF NOT EXISTS idx_documents_user_id ON documents(user_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_documents_created_at ON documents(created_at)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_activities_user_id ON activities(user_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_activities_created_at ON activities(created_at)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)", [])?;
        
        // ÍNDICES PARA TRILHA DE AUDITORIA - OTIMIZADOS PARA SEQUÊNCIA
        conn.execute("CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs(user_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_audit_logs_resource ON audit_logs(resource_type, resource_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_audit_logs_current_hash ON audit_logs(current_hash)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_audit_logs_sequence_id ON audit_logs(sequence_id)", [])?;
        
        // ==================================================================================
        // SISTEMA FTS5 COMPLETO - BUSCA FULL-TEXT DE ALTA PERFORMANCE
        // ==================================================================================
        
        // Tabela de conteúdo dos documentos (extraído via OCR)
        conn.execute(r#"
            CREATE TABLE IF NOT EXISTS document_content (
                document_id TEXT PRIMARY KEY,
                extracted_text TEXT NOT NULL DEFAULT '',
                document_type TEXT NOT NULL DEFAULT 'generic',
                extracted_fields TEXT NOT NULL DEFAULT '{}',
                indexed_at TEXT NOT NULL,
                FOREIGN KEY (document_id) REFERENCES documents (id) ON DELETE CASCADE
            )
        "#, [])?;
        
        // TABELA VIRTUAL FTS5 - MOTOR DE BUSCA FULL-TEXT
        // Usando configuração otimizada para performance máxima
        conn.execute(r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
                document_id UNINDEXED,
                extracted_text,
                document_type UNINDEXED,
                extracted_fields,
                content='document_content',
                content_rowid='document_id',
                tokenize='unicode61 remove_diacritics 1'
            )
        "#, [])?;
        
        // TRIGGERS CRÍTICOS - SINCRONIZAÇÃO AUTOMÁTICA FTS5
        // Inserção automática no FTS5 quando conteúdo é adicionado
        conn.execute(r#"
            CREATE TRIGGER IF NOT EXISTS documents_fts_insert 
            AFTER INSERT ON document_content 
            BEGIN
                INSERT INTO documents_fts(document_id, extracted_text, document_type, extracted_fields)
                VALUES (NEW.document_id, NEW.extracted_text, NEW.document_type, NEW.extracted_fields);
            END
        "#, [])?;
        
        // Atualização automática do FTS5 quando conteúdo é modificado
        conn.execute(r#"
            CREATE TRIGGER IF NOT EXISTS documents_fts_update
            AFTER UPDATE ON document_content
            BEGIN
                UPDATE documents_fts 
                SET extracted_text = NEW.extracted_text,
                    document_type = NEW.document_type,
                    extracted_fields = NEW.extracted_fields
                WHERE document_id = NEW.document_id;
            END
        "#, [])?;
        
        // Remoção automática do FTS5 quando conteúdo é deletado
        conn.execute(r#"
            CREATE TRIGGER IF NOT EXISTS documents_fts_delete
            AFTER DELETE ON document_content
            BEGIN
                DELETE FROM documents_fts WHERE document_id = OLD.document_id;
            END
        "#, [])?;
        
        // ÍNDICES PARA PERFORMANCE DE BUSCA
        conn.execute("CREATE INDEX IF NOT EXISTS idx_document_content_document_id ON document_content(document_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_document_content_document_type ON document_content(document_type)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_document_content_indexed_at ON document_content(indexed_at)", [])?;
        
        log::info!("📊 Schema FTS5 criado com sucesso - busca full-text ativada");
        
        // INICIALIZAR CONFIGURAÇÃO FTS5 (se necessário)
        // Rebuild do índice FTS5 caso exista conteúdo sem indexação
        let rebuild_result = conn.execute("INSERT INTO documents_fts(documents_fts) VALUES('rebuild')", []);
        match rebuild_result {
            Ok(_) => log::info!("🔧 Índice FTS5 reconstruído"),
            Err(_) => log::debug!("📝 Índice FTS5 não necessita reconstrução")
        }
        
        Ok(())
    }
    
    // OPERAÇÃO COM RETRY AUTOMÁTICO - CORRIGIDO
    fn execute_with_retry<F, R>(&self, operation: F) -> SqliteResult<R> 
    where
        F: Fn(&Connection) -> SqliteResult<R>,
    {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY_MS: u64 = 100;
        
        let mut last_error = None;
        
        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                thread::sleep(Duration::from_millis(RETRY_DELAY_MS * (attempt as u64)));
            }
            
            match self.conn.lock() {
                Ok(conn) => {
                    match operation(&*conn) {
                        Ok(result) => return Ok(result),
                        Err(e) => {
                            last_error = Some(e);
                            // Se for erro de busy/lock, tenta novamente
                            if let Some(rusqlite::Error::SqliteFailure(err, _)) = last_error.as_ref() {
                                if err.code == rusqlite::ErrorCode::DatabaseBusy || 
                                   err.code == rusqlite::ErrorCode::DatabaseLocked {
                                    continue;
                                }
                            }
                            // Para outros erros, falha imediatamente
                            return Err(last_error.unwrap());
                        }
                    }
                },
                Err(_) => {
                    // Se não conseguir lock do Mutex, espera e tenta novamente
                    continue;
                }
            }
        }
        
        // CORREÇÃO CRÍTICA: Conversão correta para i32
        Err(last_error.unwrap_or_else(|| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ErrorCode::DatabaseBusy as i32),
                Some("Max retries exceeded".to_string())
            )
        }))
    }
    
    pub fn create_user(&self, user: &User) -> SqliteResult<()> {
        self.execute_with_retry(|conn| {
            conn.execute(
                "INSERT INTO users (id, username, email, password_hash, created_at, last_login) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    user.id,
                    user.username,
                    user.email,
                    user.password_hash,
                    user.created_at.to_rfc3339(),
                    user.last_login.map(|dt| dt.to_rfc3339())
                ]
            )?;
            Ok(())
        })
    }
    
    pub fn get_user_by_username(&self, username: &str) -> SqliteResult<Option<User>> {
        self.execute_with_retry(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, username, email, password_hash, created_at, last_login FROM users WHERE username = ?1"
            )?;
            
            let user_iter = stmt.query_map([username], |row| {
                let created_at_str: String = row.get(4)?;
                let last_login_str: Option<String> = row.get(5)?;
                
                Ok(User {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    email: row.get(2)?,
                    password_hash: row.get(3)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map_err(|_| rusqlite::Error::InvalidColumnType(4, "created_at".to_string(), rusqlite::types::Type::Text))?
                        .with_timezone(&Utc),
                    last_login: last_login_str.map(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now())
                    }),
                })
            })?;
            
            for user in user_iter {
                return Ok(Some(user?));
            }
            
            Ok(None)
        })
    }
    
    pub fn create_document(&self, document: &Document) -> SqliteResult<()> {
        self.execute_with_retry(|conn| {
            let tags_json = serde_json::to_string(&document.tags)
                .map_err(|_| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to serialize tags"))))?;
                
            conn.execute(
                "INSERT INTO documents (id, user_id, name, file_path, file_type, file_size, created_at, updated_at, tags) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    document.id,
                    document.user_id,
                    document.name,
                    document.file_path,
                    document.file_type,
                    document.file_size,
                    document.created_at.to_rfc3339(),
                    document.updated_at.to_rfc3339(),
                    tags_json
                ]
            )?;
            Ok(())
        })
    }
    
    pub fn get_documents_by_user(&self, user_id: &str) -> SqliteResult<Vec<Document>> {
        self.execute_with_retry(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, user_id, name, file_path, file_type, file_size, created_at, updated_at, tags FROM documents WHERE user_id = ?1 ORDER BY created_at DESC"
            )?;
            
            let document_iter = stmt.query_map([user_id], |row| {
                let created_at_str: String = row.get(6)?;
                let updated_at_str: String = row.get(7)?;
                let tags_json: String = row.get(8)?;
                
                let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
                
                Ok(Document {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    name: row.get(2)?,
                    file_path: row.get(3)?,
                    file_type: row.get(4)?,
                    file_size: row.get(5)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map_err(|_| rusqlite::Error::InvalidColumnType(6, "created_at".to_string(), rusqlite::types::Type::Text))?
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                        .map_err(|_| rusqlite::Error::InvalidColumnType(7, "updated_at".to_string(), rusqlite::types::Type::Text))?
                        .with_timezone(&Utc),
                    tags,
                })
            })?;
            
            let mut documents = Vec::new();
            for document in document_iter {
                documents.push(document?);
            }
            
            Ok(documents)
        })
    }
    
    pub fn log_activity(&self, activity: &Activity) -> SqliteResult<()> {
        self.execute_with_retry(|conn| {
            conn.execute(
                "INSERT INTO activities (id, user_id, action, resource_type, resource_id, details, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    activity.id,
                    activity.user_id,
                    activity.action,
                    activity.resource_type,
                    activity.resource_id,
                    activity.details,
                    activity.created_at.to_rfc3339()
                ]
            )?;
            Ok(())
        })
    }
    
    pub fn get_user_stats(&self, user_id: &str) -> SqliteResult<(i64, i64, i64)> {
        self.execute_with_retry(|conn| {
            // Total de documentos
            let mut stmt = conn.prepare("SELECT COUNT(*) FROM documents WHERE user_id = ?1")?;
            let document_count: i64 = stmt.query_row([user_id], |row| row.get(0))?;
            
            // Total de atividades
            let mut stmt = conn.prepare("SELECT COUNT(*) FROM activities WHERE user_id = ?1")?;
            let activity_count: i64 = stmt.query_row([user_id], |row| row.get(0))?;
            
            // Tamanho total dos arquivos
            let mut stmt = conn.prepare("SELECT COALESCE(SUM(file_size), 0) FROM documents WHERE user_id = ?1")?;
            let total_size: i64 = stmt.query_row([user_id], |row| row.get(0))?;
            
            Ok((document_count, activity_count, total_size))
        })
    }
    
    // COMANDO PARA VERIFICAR INTEGRIDADE DO BANCO
    pub fn verify_integrity(&self) -> SqliteResult<bool> {
        self.execute_with_retry(|conn| {
            let mut stmt = conn.prepare("PRAGMA integrity_check")?;
            let result: String = stmt.query_row([], |row| row.get(0))?;
            Ok(result == "ok")
        })
    }
    
    // OTIMIZAR BANCO (VACUUM + ANALYZE)
    pub fn optimize(&self) -> SqliteResult<()> {
        self.execute_with_retry(|conn| {
            conn.execute("PRAGMA optimize", [])?;
            conn.execute("ANALYZE", [])?;
            Ok(())
        })
    }
    
    // CHECKPOINT MANUAL DO WAL
    pub fn checkpoint(&self) -> SqliteResult<()> {
        self.execute_with_retry(|conn| {
            conn.execute("PRAGMA wal_checkpoint(TRUNCATE)", [])?;
            Ok(())
        })
    }

    // ================================
    // SISTEMA DE TRILHA DE AUDITORIA LEGAL
    // ================================
    
    // Calcular hash SHA-256 para integridade criptográfica
    fn calculate_hash(&self, data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let result = hasher.finalize();
        let mut hash_string = String::new();
        for byte in result {
            write!(&mut hash_string, "{:02x}", byte).unwrap();
        }
        hash_string
    }
    
    // Obter o último hash da cadeia (blockchain-like) - TRANSACIONAL PARA EVITAR RACE CONDITIONS
    fn get_last_audit_hash(&self, conn: &Connection) -> SqliteResult<String> {
        // Usar transação IMMEDIATE para evitar problemas de concorrência
        conn.execute("BEGIN IMMEDIATE", [])?;
        
        let result = match conn.prepare("SELECT current_hash FROM audit_logs ORDER BY sequence_id DESC LIMIT 1") {
            Ok(mut stmt) => {
                match stmt.query_row([], |row| {
                    let hash: String = row.get(0)?;
                    Ok(hash)
                }) {
                    Ok(hash) => Ok(hash),
                    Err(rusqlite::Error::QueryReturnedNoRows) => {
                        // Primeiro log - usar hash inicial
                        Ok("0000000000000000000000000000000000000000000000000000000000000000".to_string())
                    }
                    Err(e) => Err(e)
                }
            }
            Err(e) => Err(e)
        };
        
        // Não fazer COMMIT aqui - será feito pela função que criou a transação
        result
    }
    
    // FUNÇÃO PRINCIPAL: Criar log de auditoria imutável - PROTEGIDA CONTRA RACE CONDITIONS
    pub fn create_audit_log(
        &self,
        user_id: &str,
        username: &str,
        action: &str,
        resource_type: &str,
        resource_id: Option<String>,
        resource_name: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        file_hash: Option<String>,
        metadata: Option<serde_json::Value>,
        is_success: bool,
    ) -> SqliteResult<AuditLog> {
        self.execute_with_retry(|conn| {
            // TRANSAÇÃO ATÔMICA PARA EVITAR RACE CONDITIONS NA CADEIA DE HASH
            let log_id = Uuid::new_v4().to_string();
            let timestamp = Utc::now();
            
            // Obter último hash dentro da mesma transação
            let previous_hash = self.get_last_audit_hash(conn)?;
            
            // Criar string para hash (determinística)
            let metadata_str = metadata
                .map(|m| m.to_string())
                .unwrap_or_else(|| "{}".to_string());
                
            let hash_data = format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                log_id,
                user_id,
                username,
                action,
                resource_type,
                resource_id.as_deref().unwrap_or(""),
                resource_name.as_deref().unwrap_or(""),
                ip_address.as_deref().unwrap_or(""),
                file_hash.as_deref().unwrap_or(""),
                previous_hash,
                metadata_str,
                timestamp.to_rfc3339(),
                is_success
            );
            
            let current_hash = self.calculate_hash(&hash_data);
            
            // Inserir no banco (sequence_id será auto-gerado)
            conn.execute(
                r#"INSERT INTO audit_logs 
                   (id, user_id, username, action, resource_type, resource_id, resource_name, 
                    ip_address, user_agent, file_hash, previous_hash, current_hash, metadata, 
                    timestamp, is_success) 
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)"#,
                params![
                    log_id,
                    user_id,
                    username,
                    action,
                    resource_type,
                    resource_id,
                    resource_name,
                    ip_address,
                    user_agent,
                    file_hash,
                    previous_hash,
                    current_hash,
                    metadata_str,
                    timestamp.to_rfc3339(),
                    is_success
                ]
            )?;
            
            // Obter o sequence_id gerado
            let sequence_id = conn.last_insert_rowid();
            
            // COMMIT da transação
            conn.execute("COMMIT", [])?;
            
            Ok(AuditLog {
                sequence_id,
                id: log_id,
                user_id: user_id.to_string(),
                username: username.to_string(),
                action: action.to_string(),
                resource_type: resource_type.to_string(),
                resource_id,
                resource_name,
                ip_address,
                user_agent,
                file_hash,
                previous_hash,
                current_hash,
                metadata: metadata_str,
                timestamp,
                is_success,
            })
        })
    }
    
    // Buscar logs de auditoria com filtros
    pub fn get_audit_logs(
        &self,
        user_id: Option<&str>,
        action: Option<&str>,
        resource_type: Option<&str>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> SqliteResult<Vec<AuditLog>> {
        self.execute_with_retry(|conn| {
            let mut query = "SELECT sequence_id, id, user_id, username, action, resource_type, resource_id, resource_name, ip_address, user_agent, file_hash, previous_hash, current_hash, metadata, timestamp, is_success FROM audit_logs WHERE 1=1".to_string();
            let mut params = Vec::new();
            
            if let Some(uid) = user_id {
                query.push_str(" AND user_id = ?");
                params.push(uid);
            }
            
            if let Some(act) = action {
                query.push_str(" AND action = ?");
                params.push(act);
            }
            
            if let Some(rt) = resource_type {
                query.push_str(" AND resource_type = ?");
                params.push(rt);
            }
            
            if let Some(start) = start_date {
                query.push_str(" AND timestamp >= ?");
                params.push(&start.to_rfc3339());
            }
            
            if let Some(end) = end_date {
                query.push_str(" AND timestamp <= ?");
                params.push(&end.to_rfc3339());
            }
            
            query.push_str(" ORDER BY sequence_id DESC");
            
            if let Some(lim) = limit {
                query.push_str(&format!(" LIMIT {}", lim));
            }
            
            let mut stmt = conn.prepare(&query)?;
            
            // Converter para rusqlite::types::Value
            let sqlite_params: Vec<&dyn rusqlite::ToSql> = params.iter()
                .map(|p| p as &dyn rusqlite::ToSql)
                .collect();
            
            let audit_iter = stmt.query_map(&sqlite_params[..], |row| {
                let timestamp_str: String = row.get(14)?;
                
                Ok(AuditLog {
                    sequence_id: row.get(0)?,
                    id: row.get(1)?,
                    user_id: row.get(2)?,
                    username: row.get(3)?,
                    action: row.get(4)?,
                    resource_type: row.get(5)?,
                    resource_id: row.get(6)?,
                    resource_name: row.get(7)?,
                    ip_address: row.get(8)?,
                    user_agent: row.get(9)?,
                    file_hash: row.get(10)?,
                    previous_hash: row.get(11)?,
                    current_hash: row.get(12)?,
                    metadata: row.get(13)?,
                    timestamp: DateTime::parse_from_rfc3339(&timestamp_str)
                        .map_err(|_| rusqlite::Error::InvalidColumnType(14, "timestamp".to_string(), rusqlite::types::Type::Text))?
                        .with_timezone(&Utc),
                    is_success: row.get(15)?,
                })
            })?;
            
            let mut logs = Vec::new();
            for log in audit_iter {
                logs.push(log?);
            }
            
            Ok(logs)
        })
    }
    
    // Verificar integridade completa da cadeia de auditoria - CRIPTOGRAFICAMENTE SEGURA
    pub fn verify_audit_chain(&self) -> SqliteResult<bool> {
        self.execute_with_retry(|conn| {
            // Usar sequence_id para garanta de ordem monotonica
            let mut stmt = conn.prepare("SELECT sequence_id, id, user_id, username, action, resource_type, resource_id, resource_name, ip_address, user_agent, file_hash, previous_hash, current_hash, metadata, timestamp, is_success FROM audit_logs ORDER BY sequence_id ASC")?;
            
            let audit_iter = stmt.query_map([], |row| {
                let timestamp_str: String = row.get(14)?;
                Ok(AuditLog {
                    sequence_id: row.get(0)?,
                    id: row.get(1)?,
                    user_id: row.get(2)?,
                    username: row.get(3)?,
                    action: row.get(4)?,
                    resource_type: row.get(5)?,
                    resource_id: row.get(6)?,
                    resource_name: row.get(7)?,
                    ip_address: row.get(8)?,
                    user_agent: row.get(9)?,
                    file_hash: row.get(10)?,
                    previous_hash: row.get(11)?,
                    current_hash: row.get(12)?,
                    metadata: row.get(13)?,
                    timestamp: DateTime::parse_from_rfc3339(&timestamp_str)
                        .map_err(|_| rusqlite::Error::InvalidColumnType(14, "timestamp".to_string(), rusqlite::types::Type::Text))?
                        .with_timezone(&Utc),
                    is_success: row.get(15)?,
                })
            })?;
            
            let mut previous_hash = "0000000000000000000000000000000000000000000000000000000000000000".to_string();
            let mut expected_sequence = 1i64; // Primeiro sequence_id deve ser 1
            
            for log_result in audit_iter {
                let log = log_result?;
                
                // VERIFICAÇÃO CRÍTICA 1: Sequence ID deve ser consecutivo
                if log.sequence_id != expected_sequence {
                    log::error!("FALHA AUDITORIA: Sequence ID inválido. Esperado: {}, Encontrado: {}", expected_sequence, log.sequence_id);
                    return Ok(false);
                }
                
                // VERIFICAÇÃO CRÍTICA 2: Hash anterior deve coincidir
                if log.previous_hash != previous_hash {
                    log::error!("FALHA AUDITORIA: Previous hash inválido no sequence_id {}", log.sequence_id);
                    return Ok(false);
                }
                
                // VERIFICAÇÃO CRÍTICA 3: Recalcular hash e verificar integridade
                let hash_data = format!(
                    "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                    log.id,
                    log.user_id,
                    log.username,
                    log.action,
                    log.resource_type,
                    log.resource_id.as_deref().unwrap_or(""),
                    log.resource_name.as_deref().unwrap_or(""),
                    log.ip_address.as_deref().unwrap_or(""),
                    log.file_hash.as_deref().unwrap_or(""),
                    log.previous_hash,
                    log.metadata,
                    log.timestamp.to_rfc3339(),
                    log.is_success
                );
                
                let calculated_hash = self.calculate_hash(&hash_data);
                if calculated_hash != log.current_hash {
                    log::error!("FALHA AUDITORIA: Hash calculado difere do armazenado no sequence_id {}", log.sequence_id);
                    return Ok(false);
                }
                
                previous_hash = log.current_hash;
                expected_sequence += 1;
            }
            
            log::info!("SUCESSO: Trilha de auditoria íntegra. Verificados {} registros.", expected_sequence - 1);
            Ok(true)
        })
    }
    
    // NOVA FUNÇÃO: Estatísticas da trilha de auditoria
    pub fn get_audit_chain_stats(&self) -> SqliteResult<(usize, Option<String>, Option<String>)> {
        self.execute_with_retry(|conn| {
            // Total de logs
            let mut stmt = conn.prepare("SELECT COUNT(*) FROM audit_logs")?;
            let total_logs: usize = stmt.query_row([], |row| Ok(row.get::<_, i64>(0)? as usize))?;
            
            if total_logs == 0 {
                return Ok((0, None, None));
            }
            
            // Primeiro log
            let mut stmt = conn.prepare("SELECT timestamp FROM audit_logs ORDER BY sequence_id ASC LIMIT 1")?;
            let first_log: String = stmt.query_row([], |row| row.get(0))?;
            
            // Último log
            let mut stmt = conn.prepare("SELECT timestamp FROM audit_logs ORDER BY sequence_id DESC LIMIT 1")?;
            let last_log: String = stmt.query_row([], |row| row.get(0))?;
            
            Ok((total_logs, Some(first_log), Some(last_log)))
        })
    }
    
    // ================================
    // SISTEMA DE BUSCA FULL-TEXT FTS5
    // ================================
    
    // Indexar conteúdo de documento para busca
    pub fn index_document_content(
        &self,
        document_id: &str,
        extracted_text: &str,
        document_type: &str,
        extracted_fields: &serde_json::Value,
    ) -> SqliteResult<()> {
        self.execute_with_retry(|conn| {
            let fields_json = extracted_fields.to_string();
            let indexed_at = Utc::now().to_rfc3339();
            
            // Inserir ou atualizar conteúdo
            conn.execute(
                r#"INSERT OR REPLACE INTO document_content 
                   (document_id, extracted_text, document_type, extracted_fields, indexed_at) 
                   VALUES (?1, ?2, ?3, ?4, ?5)"#,
                params![document_id, extracted_text, document_type, fields_json, indexed_at]
            )?;
            
            log::info!("📝 Documento {} indexado para busca ({} caracteres)", 
                      document_id, extracted_text.len());
            Ok(())
        })
    }
    
    // Busca full-text nos documentos
    pub fn search_documents(
        &self,
        user_id: &str,
        query: &str,
        limit: Option<usize>,
    ) -> SqliteResult<Vec<SearchResult>> {
        self.execute_with_retry(|conn| {
            let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or(" LIMIT 50".to_string());
            
            // Query FTS5 com ranking
            let search_query = format!(
                r#"SELECT 
                    fts.document_id,
                    d.name as document_name,
                    fts.document_type,
                    d.file_path,
                    bm25(fts) as relevance_score,
                    snippet(fts, 2, '<mark>', '</mark>', '...', 64) as matched_content,
                    d.created_at
                   FROM documents_fts fts
                   JOIN documents d ON d.id = fts.document_id
                   WHERE d.user_id = ?1 AND fts MATCH ?2
                   ORDER BY relevance_score ASC{}"#,
                limit_clause
            );
            
            let mut stmt = conn.prepare(&search_query)?;
            let search_iter = stmt.query_map([user_id, query], |row| {
                let created_at_str: String = row.get(6)?;
                Ok(SearchResult {
                    document_id: row.get(0)?,
                    document_name: row.get(1)?,
                    document_type: row.get(2)?,
                    file_path: row.get(3)?,
                    relevance_score: row.get::<_, f64>(4)? as f32,
                    matched_content: row.get(5)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map_err(|_| rusqlite::Error::InvalidColumnType(6, "created_at".to_string(), rusqlite::types::Type::Text))?
                        .with_timezone(&Utc),
                })
            })?;
            
            let mut results = Vec::new();
            for result in search_iter {
                results.push(result?);
            }
            
            log::info!("🔍 Busca '{}' retornou {} resultados para user {}", 
                      query, results.len(), user_id);
            Ok(results)
        })
    }
    
    // Busca simples nos documentos (fallback se FTS5 não disponível)
    pub fn simple_search_documents(
        &self,
        user_id: &str,
        query: &str,
        limit: Option<usize>,
    ) -> SqliteResult<Vec<SearchResult>> {
        self.execute_with_retry(|conn| {
            let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or(" LIMIT 50".to_string());
            
            // Query simples com LIKE
            let search_query = format!(
                r#"SELECT 
                    d.id,
                    d.name,
                    COALESCE(dc.document_type, 'Generico') as document_type,
                    d.file_path,
                    1.0 as relevance_score,
                    SUBSTR(COALESCE(dc.extracted_text, d.name), 1, 200) as matched_content,
                    d.created_at
                   FROM documents d
                   LEFT JOIN document_content dc ON dc.document_id = d.id
                   WHERE d.user_id = ?1 
                   AND (d.name LIKE ?2 OR dc.extracted_text LIKE ?2 OR dc.extracted_fields LIKE ?2)
                   ORDER BY d.created_at DESC{}"#,
                limit_clause
            );
            
            let like_query = format!("%{}%", query);
            let mut stmt = conn.prepare(&search_query)?;
            let search_iter = stmt.query_map([user_id, &like_query], |row| {
                let created_at_str: String = row.get(6)?;
                Ok(SearchResult {
                    document_id: row.get(0)?,
                    document_name: row.get(1)?,
                    document_type: row.get(2)?,
                    file_path: row.get(3)?,
                    relevance_score: row.get::<_, f64>(4)? as f32,
                    matched_content: row.get(5)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map_err(|_| rusqlite::Error::InvalidColumnType(6, "created_at".to_string(), rusqlite::types::Type::Text))?
                        .with_timezone(&Utc),
                })
            })?;
            
            let mut results = Vec::new();
            for result in search_iter {
                results.push(result?);
            }
            
            Ok(results)
        })
    }
    
    // Estatísticas de busca
    pub fn get_search_stats(&self, user_id: &str) -> SqliteResult<(i64, i64)> {
        self.execute_with_retry(|conn| {
            // Total de documentos do usuário
            let mut stmt = conn.prepare("SELECT COUNT(*) FROM documents WHERE user_id = ?1")?;
            let total_docs: i64 = stmt.query_row([user_id], |row| row.get(0))?;
            
            // Documentos indexados
            let mut stmt = conn.prepare(
                "SELECT COUNT(*) FROM document_content dc 
                 JOIN documents d ON d.id = dc.document_id 
                 WHERE d.user_id = ?1"
            )?;
            let indexed_docs: i64 = stmt.query_row([user_id], |row| row.get(0)).unwrap_or(0);
            
            Ok((total_docs, indexed_docs))
        })
    }
    
    // Recriar índices FTS5 (manutenção)
    pub fn rebuild_search_index(&self) -> SqliteResult<()> {
        self.execute_with_retry(|conn| {
            conn.execute("INSERT INTO documents_fts(documents_fts) VALUES('rebuild')", [])?;
            log::info!("🔄 Índice de busca FTS5 reconstruído");
            Ok(())
        })
    }
}
