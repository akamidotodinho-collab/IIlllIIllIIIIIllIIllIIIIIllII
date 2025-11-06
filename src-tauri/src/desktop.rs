use tauri::{command, AppHandle};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use std::path::PathBuf;

// Abrir diálogo de seleção de arquivos nativos
#[command]
pub async fn open_file_dialog(app: AppHandle) -> Result<Vec<String>, String> {
    let files = app.dialog()
        .file()
        .add_filter("Documentos", &["pdf", "doc", "docx", "xls", "xlsx"])
        .add_filter("Imagens", &["jpg", "jpeg", "png", "gif", "bmp", "webp"])
        .add_filter("Todos", &["*"])
        .set_title("Selecionar documentos")
        .blocking_pick_files();
    
    match files {
        Some(files) => {
            let paths: Vec<String> = files
                .into_iter()
                .filter_map(|p| p.path.to_str().map(|s| s.to_string()))
                .collect();
            Ok(paths)
        },
        None => Ok(vec![])
    }
}

// Abrir diálogo para salvar backup
#[command]
pub async fn save_backup_dialog(app: AppHandle) -> Result<Option<String>, String> {
    let file_name = format!("arkive-backup-{}.zip", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    
    let path = app.dialog()
        .file()
        .add_filter("Backup ARKIVE", &["zip"])
        .set_file_name(&file_name)
        .set_title("Salvar backup")
        .blocking_save_file();
    
    Ok(path.and_then(|p| p.path.to_str().map(|s| s.to_string())))
}

// Abrir pasta no explorador
#[command]
pub async fn open_in_explorer(path: String) -> Result<(), String> {
    let path = PathBuf::from(path);
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Erro ao abrir explorador: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Erro ao abrir finder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Erro ao abrir gerenciador: {}", e))?;
    }
    
    Ok(())
}

// Notificação nativa do sistema
#[command]
pub async fn show_notification(title: String, message: String) -> Result<(), String> {
    println!("Notificação: {} - {}", title, message);
    Ok(())
}
