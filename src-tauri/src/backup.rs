use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Read};
use zip::{ZipArchive, result::ZipError};
use rusqlite::Connection;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

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
        
     {
    let mut backup_info_file = archive.by_name("backup_info.json")?;
    let mut backup_info_content = String::new();
    backup_info_file.read_to_string(&mut backup_info_content)?;
} // fim do escopo de backup_info_file

let mut db_file = archive.by_name("database.db")?;

        
        let backup_info: BackupInfo = serde_json::from_str(&backup_info_content)
            .map_err(|e| BackupError::ValidationError(format!("JSON inválido: {}", e)))?;
        
        // Validar banco de dados extraindo-o temporariamente
        let temp_dir = tempfile::tempdir()?;
        let temp_db_path = temp_dir.path().join("temp_database.db");
        
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
}

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
