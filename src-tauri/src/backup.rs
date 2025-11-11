use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Read, Write};
use zip::{ZipArchive, ZipWriter, result::ZipError, write::SimpleFileOptions, CompressionMethod};
use rusqlite::Connection;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupInfo {
    pub created_at: DateTime<Utc>,
    pub version: String,
    pub database_size: u64,
    pub files_count: usize,
    pub checksum: String,
}

#[derive(Debug)]
pub enum BackupError {
    IoError(io::Error),
    ZipError(ZipError),
    DatabaseError(rusqlite::Error),
    ValidationError(String),
}

impl From<io::Error> for BackupError {
    fn from(error: io::Error) -> Self {
        BackupError::IoError(error)
    }
}

impl From<ZipError> for BackupError {
    fn from(error: ZipError) -> Self {
        BackupError::ZipError(error)
    }
}

impl From<rusqlite::Error> for BackupError {
    fn from(error: rusqlite::Error) -> Self {
        BackupError::DatabaseError(error)
    }
}

pub struct BackupManager {
    backup_dir: PathBuf,
}

impl BackupManager {
    pub fn new(backup_dir: PathBuf) -> Self {
        Self { backup_dir }
    }
    
    /// Verifica a integridade de um arquivo de backup
    pub fn verify_backup(&self, backup_path: &Path) -> Result<BackupInfo, BackupError> {
        if !backup_path.exists() {
            return Err(BackupError::ValidationError("Arquivo de backup não encontrado".to_string()));
        }
        
        // Verificar se é um arquivo ZIP válido
        let file = fs::File::open(backup_path)?;
        let mut archive = ZipArchive::new(file)?;
        
        // Verificar se contém os arquivos essenciais
        let required_files = vec!["database.db", "backup_info.json"];
        let mut found_files = Vec::new();
        
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            found_files.push(file.name().to_string());
        }
        
        for required in &required_files {
            if !found_files.iter().any(|f| f.contains(required)) {
                return Err(BackupError::ValidationError(
                    format!("Arquivo obrigatório '{}' não encontrado no backup", required)
                ));
            }
        }
        
        // CORREÇÃO CRÍTICA: Ler backup_info.json em bloco separado
        let backup_info: BackupInfo = {
            let mut backup_info_file = archive.by_name("backup_info.json")?;
            let mut backup_info_content = String::new();
            backup_info_file.read_to_string(&mut backup_info_content)?;
            
            serde_json::from_str(&backup_info_content)
                .map_err(|e| BackupError::ValidationError(format!("JSON inválido: {}", e)))?
        };
        // backup_info_file saiu de escopo aqui - borrow foi liberado
        
        // Validar banco de dados extraindo-o temporariamente
        let temp_dir = tempfile::tempdir()?;
        let temp_db_path = temp_dir.path().join("temp_database.db");
        
        // Agora é seguro abrir outro arquivo do archive
        let mut db_file = archive.by_name("database.db")?;
        let mut temp_db_file = fs::File::create(&temp_db_path)?;
        io::copy(&mut db_file, &mut temp_db_file)?;
        
        // Verificar integridade do SQLite
        let temp_conn = Connection::open(&temp_db_path)?;
        let integrity_result: String = temp_conn.query_row("PRAGMA integrity_check", [], |row| {
            row.get(0)
        })?;
        
        if integrity_result != "ok" {
            return Err(BackupError::ValidationError(
                format!("Banco de dados corrompido: {}", integrity_result)
            ));
        }
        
        // Verificar estrutura das tabelas
        let tables: Vec<String> = temp_conn.prepare("SELECT name FROM sqlite_master WHERE type='table'")?
            .query_map([], |row| Ok(row.get::<_, String>(0)?))?
            .collect::<Result<Vec<_>, _>>()?;
        
        let required_tables = vec!["users", "documents", "activities"];
        for table in &required_tables {
            if !tables.contains(&table.to_string()) {
                return Err(BackupError::ValidationError(
                    format!("Tabela obrigatória '{}' não encontrada", table)
                ));
            }
        }
        
        println!("✅ Backup verificado com sucesso!");
        println!("   - Data: {}", backup_info.created_at.format("%d/%m/%Y %H:%M"));
        println!("   - Versão: {}", backup_info.version);
        println!("   - Tamanho DB: {} bytes", backup_info.database_size);
        println!("   - Arquivos: {}", backup_info.files_count);
        println!("   - Checksum: {}", &backup_info.checksum[..8]);
        
        Ok(backup_info)
    }
    
    /// Lista todos os backups disponíveis
    pub fn list_backups(&self) -> Result<Vec<(PathBuf, BackupInfo)>, BackupError> {
        if !self.backup_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut backups = Vec::new();
        
        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("zip") {
                match self.verify_backup(&path) {
                    Ok(info) => backups.push((path, info)),
                    Err(e) => {
                        println!("⚠️  Backup inválido {}: {:?}", path.display(), e);
                    }
                }
            }
        }
        
        // Ordenar por data (mais recente primeiro)
        backups.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));
        
        Ok(backups)
    }
    
    /// Limpar backups antigos (manter apenas os N mais recentes)
    pub fn cleanup_old_backups(&self, keep_count: usize) -> Result<usize, BackupError> {
        let backups = self.list_backups()?;
        
        if backups.len() <= keep_count {
            return Ok(0);
        }
        
        let mut removed_count = 0;
        for (path, info) in backups.iter().skip(keep_count) {
            println!("🗑️  Removendo backup antigo: {} ({})", 
                     path.file_name().unwrap().to_string_lossy(),
                     info.created_at.format("%d/%m/%Y"));
            
            fs::remove_file(path)?;
            removed_count += 1;
        }
        
        Ok(removed_count)
    }
    
    /// Criar um novo backup completo
    pub fn create_backup(
        &self,
        db_path: &Path,
        files_dir: &Path,
        output_path: &Path,
    ) -> Result<BackupInfo, BackupError> {
        println!("📦 Iniciando criação de backup...");
        
        if !db_path.exists() {
            return Err(BackupError::ValidationError("Banco de dados não encontrado".to_string()));
        }
        
        let output_file = fs::File::create(output_path)?;
        let mut zip = ZipWriter::new(output_file);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(6));
        
        let mut files_count = 0;
        let mut hasher = Sha256::new();
        
        println!("📄 Adicionando database.db ao backup...");
        let mut db_file = fs::File::open(db_path)?;
        zip.start_file("database.db", options)?;
        let db_size = io::copy(&mut db_file, &mut zip)?;
        
        hasher.update(db_size.to_le_bytes());
        files_count += 1;
        
        println!("📁 Adicionando arquivos do usuário ao backup...");
        if files_dir.exists() && files_dir.is_dir() {
            let files_added = Self::add_directory_to_zip(&mut zip, files_dir, "files", options, &mut hasher)?;
            files_count += files_added;
            println!("   ✅ {} arquivos adicionados", files_added);
        } else {
            println!("   ℹ️  Nenhum diretório de arquivos encontrado");
        }
        
        let created_at = Utc::now();
        let backup_info = BackupInfo {
            created_at,
            version: env!("CARGO_PKG_VERSION").to_string(),
            database_size: db_size,
            files_count,
            checksum: format!("{:x}", hasher.finalize()),
        };
        
        println!("📋 Adicionando metadados ao backup...");
        let backup_info_json = serde_json::to_string_pretty(&backup_info)
            .map_err(|e| BackupError::ValidationError(format!("Erro ao serializar JSON: {}", e)))?;
        
        zip.start_file("backup_info.json", options)?;
        zip.write_all(backup_info_json.as_bytes())?;
        
        zip.finish()?;
        
        println!("✅ Backup criado com sucesso!");
        println!("   - Local: {}", output_path.display());
        println!("   - Tamanho DB: {} bytes", db_size);
        println!("   - Total de arquivos: {}", files_count);
        println!("   - Checksum: {}", &backup_info.checksum[..16]);
        
        Ok(backup_info)
    }
    
    fn add_directory_to_zip<W: Write + io::Seek>(
        zip: &mut ZipWriter<W>,
        dir_path: &Path,
        prefix: &str,
        options: SimpleFileOptions,
        hasher: &mut Sha256,
    ) -> Result<usize, BackupError> {
        let mut count = 0;
        
        if !dir_path.exists() {
            return Ok(0);
        }
        
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name();
            let zip_path = format!("{}/{}", prefix, name.to_string_lossy());
            
            if path.is_file() {
                let mut file = fs::File::open(&path)?;
                zip.start_file(&zip_path, options)?;
                let size = io::copy(&mut file, zip)?;
                hasher.update(size.to_le_bytes());
                count += 1;
            } else if path.is_dir() {
                count += Self::add_directory_to_zip(zip, &path, &zip_path, options, hasher)?;
            }
        }
        
        Ok(count)
    }
    
    /// Restaurar backup para um local específico
    pub fn restore_backup(
        &self,
        backup_path: &Path,
        target_db_path: &Path,
        target_files_dir: &Path,
    ) -> Result<(), BackupError> {
        println!("🔄 Iniciando restauração de backup...");
        println!("   - Origem: {}", backup_path.display());
        println!("   - Destino DB: {}", target_db_path.display());
        println!("   - Destino Files: {}", target_files_dir.display());
        
        println!("🔍 Verificando integridade do backup...");
        let backup_info = self.verify_backup(backup_path)?;
        
        println!("📦 Extraindo backup...");
        let file = fs::File::open(backup_path)?;
        let mut archive = ZipArchive::new(file)?;
        
        if let Some(parent) = target_db_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::create_dir_all(target_files_dir)?;
        
        println!("📄 Restaurando database.db...");
        {
            let mut db_file = archive.by_name("database.db")?;
            let mut output = fs::File::create(target_db_path)?;
            io::copy(&mut db_file, &mut output)?;
        }
        
        println!("📁 Restaurando arquivos do usuário...");
        let mut restored_files = 0;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_path = file.name();
            
            if file_path.starts_with("files/") && !file_path.ends_with('/') {
                let relative_path = &file_path[6..];
                let output_path = target_files_dir.join(relative_path);
                
                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                
                let mut output = fs::File::create(&output_path)?;
                io::copy(&mut file, &mut output)?;
                restored_files += 1;
            }
        }
        
        println!("🔍 Validando banco de dados restaurado...");
        let conn = Connection::open(target_db_path)?;
        let integrity_result: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        
        if integrity_result != "ok" {
            return Err(BackupError::ValidationError(
                format!("Banco de dados restaurado está corrompido: {}", integrity_result)
            ));
        }
        
        println!("✅ Backup restaurado com sucesso!");
        println!("   - Arquivos restaurados: {}", restored_files);
        println!("   - Versão do backup: {}", backup_info.version);
        println!("   - Data do backup: {}", backup_info.created_at.format("%d/%m/%Y %H:%M"));
        
        Ok(())
    }
}

// ================================
// COMANDOS TAURI PARA BACKUP
// ================================

// Comando Tauri para verificar backup
#[tauri::command]
pub fn verify_backup_file(backup_path: String) -> Result<BackupInfo, String> {
    let path = Path::new(&backup_path);
    let backup_manager = BackupManager::new(PathBuf::from("backups"));
    
    backup_manager.verify_backup(path)
        .map_err(|e| format!("Erro ao verificar backup: {:?}", e))
}

// Comando Tauri para listar backups
#[tauri::command]
pub fn list_available_backups() -> Result<Vec<(String, BackupInfo)>, String> {
    let backup_manager = BackupManager::new(PathBuf::from("backups"));
    
    backup_manager.list_backups()
        .map(|backups| {
            backups.into_iter()
                .map(|(path, info)| (path.to_string_lossy().to_string(), info))
                .collect()
        })
        .map_err(|e| format!("Erro ao listar backups: {:?}", e))
}

// Comando Tauri para criar backup
#[tauri::command]
pub async fn create_backup_command(
    backup_path: String,
    state: State<'_, crate::AppState>,
) -> Result<BackupInfo, String> {
    
    println!("🔧 Comando create_backup chamado");
    println!("   - Caminho destino: {}", backup_path);
    
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(user) = authenticated_user.as_ref() {
        println!("   - Usuário autenticado: {}", user.username);
        
        let mut data_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        data_dir.push("ARKIVE");
        
        let db_path = data_dir.join("arkive.db");
        
        let mut files_dir = data_dir.clone();
        files_dir.push("files");
        files_dir.push(&user.id);
        
        println!("   - DB Path: {}", db_path.display());
        println!("   - Files Dir: {}", files_dir.display());
        
        let mut backup_dir = data_dir.clone();
        backup_dir.push("backups");
        fs::create_dir_all(&backup_dir)
            .map_err(|e| format!("Erro ao criar diretório de backups: {:?}", e))?;
        
        let backup_manager = BackupManager::new(backup_dir);
        let output_path = Path::new(&backup_path);
        
        let result = backup_manager.create_backup(&db_path, &files_dir, output_path)
            .map_err(|e| format!("Erro ao criar backup: {:?}", e))?;
        
        println!("✅ Backup criado via comando Tauri");
        
        Ok(result)
    } else {
        Err("Usuário não autenticado".to_string())
    }
}

// Comando Tauri para restaurar backup
#[tauri::command]
pub async fn restore_backup_command(
    backup_path: String,
    state: State<'_, crate::AppState>,
) -> Result<String, String> {
    println!("🔧 Comando restore_backup chamado");
    println!("   - Caminho backup: {}", backup_path);
    
    let authenticated_user = state.authenticated_user.lock().await;
    if let Some(user) = authenticated_user.as_ref() {
        println!("   - Usuário autenticado: {}", user.username);
        
        let mut data_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        data_dir.push("ARKIVE");
        
        let db_path = data_dir.join("arkive.db");
        
        let mut files_dir = data_dir.clone();
        files_dir.push("files");
        files_dir.push(&user.id);
        
        println!("⚠️  ATENÇÃO: A restauração sobrescreverá os dados atuais!");
        println!("   - DB será sobrescrito em: {}", db_path.display());
        println!("   - Arquivos serão restaurados em: {}", files_dir.display());
        
        let mut backup_dir = data_dir.clone();
        backup_dir.push("backups");
        
        let backup_manager = BackupManager::new(backup_dir);
        let backup_file_path = Path::new(&backup_path);
        
        println!("🔒 IMPORTANTE: A conexão do banco será fechada temporariamente");
        println!("   Aguarde a conclusão da restauração...");
        
        backup_manager.restore_backup(backup_file_path, &db_path, &files_dir)
            .map_err(|e| format!("Erro ao restaurar backup: {:?}", e))?;
        
        println!("✅ Backup restaurado via comando Tauri");
        println!("⚠️  IMPORTANTE: Reinicie a aplicação para aplicar as mudanças completamente!");
        
        Ok("Backup restaurado com sucesso! Por favor, reinicie a aplicação para garantir que todas as mudanças sejam aplicadas corretamente.".to_string())
    } else {
        Err("Usuário não autenticado".to_string())
    }
}
