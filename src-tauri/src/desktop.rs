use tauri::api::dialog::FileDialogBuilder;
use tauri::{command, Window};
use std::path::PathBuf;

// Abrir diálogo de seleção de arquivos nativos
#[command]
pub async fn open_file_dialog(_window: Window) -> Result<Vec<String>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    
    FileDialogBuilder::new()
        .add_filter("Documentos", &["pdf", "doc", "docx", "xls", "xlsx"])
        .add_filter("Imagens", &["jpg", "jpeg", "png", "gif", "bmp", "webp"])
        .add_filter("Todos", &["*"])
        .set_title("Selecionar documentos")
        .pick_files(move |files| {
            let paths = files.unwrap_or_default()
                .into_iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            let _ = tx.send(paths);
        });
    
    rx.await.map_err(|e| format!("Erro ao abrir diálogo: {}", e))
}

// Abrir diálogo para salvar backup
#[command]
pub async fn save_backup_dialog(_window: Window) -> Result<Option<String>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    
    FileDialogBuilder::new()
        .add_filter("Backup ARKIVE", &["zip"])
        .set_file_name(&format!("arkive-backup-{}.zip", chrono::Utc::now().format("%Y%m%d_%H%M%S")))
        .set_title("Salvar backup")
        .save_file(move |path| {
            let result = path.map(|p| p.to_string_lossy().to_string());
            let _ = tx.send(result);
        });
    
    rx.await.map_err(|e| format!("Erro ao abrir diálogo: {}", e))
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
    // Implementar notificação nativa aqui
    println!("Notificação: {} - {}", title, message);
    Ok(())
}