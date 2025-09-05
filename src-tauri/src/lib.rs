// Adicionar estas linhas no início do arquivo
mod database_sqlite;
mod backup;

// Na função main do Tauri, adicionar os comandos
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            // ... seus comandos existentes
            backup::verify_backup_file,
            backup::list_available_backups,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
