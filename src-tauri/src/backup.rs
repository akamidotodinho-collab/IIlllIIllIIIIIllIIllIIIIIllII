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
    
    pub fn verify_backup(&self, backup_path: &Path) -> Result<BackupInfo, BackupError> {
        if !backup_path.exists() {
            return Err(BackupError::ValidationError("Arquivo de backup não encontrado".to_string()));
        }
        
        let file = fs::File::open(backup_path)?;
        let mut archive = ZipArchive::new(file)?;
        
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
        
        let mut backup_info_content = String::new();
        {
            let mut backup_info_file = archive.by_name("backup_info.json")?;
            backup_info_file.read_to_string(&mut backup_info_content)?;
        }
        
        let backup_info: BackupInfo = serde_json::from_str(&backup_info_content)
            .map_err(|e| BackupError::ValidationError(format!("JSON inválido: {}", e)))?;
        
        let temp_dir = tempfile::tempdir()?;
        let temp_db_path = temp_dir.path().join("temp_database.db");
        
        {
            let mut db_file = archive.by_name("database.db")?;
            let mut temp_db_file = fs::File::create(&temp_db_path)?;
            io::copy(&mut db_file, &mut temp_db_file)?;
        }
        
        let temp_conn = Connection::open(&temp_db_path)?;
        let integrity_result: String = temp_conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        
        if integrity_result != "ok" {
            return Err(BackupError::ValidationError(
                format!("Banco de dados corrompido: {}", integrity_result)
            ));
        }
        
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
        let display_checksum = &backup_info.checksum[..backup_info.checksum.len().min(8)];
        println!("   - Checksum: {}", display_checksum);

        Ok(backup_info)
    }

    // list_backups e cleanup_old_backups permanecem iguais...
}

