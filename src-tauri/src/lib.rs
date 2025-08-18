#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    log::info!("🏃 Iniciando função run()...");
    
    log::info!("⚙️ Criando runtime Tokio...");
    let rt = match tokio::runtime::Runtime::new() {
        Ok(runtime) => {
            log::info!("✅ Runtime Tokio criado com sucesso");
            runtime
        },
        Err(e) => {
            log::error!("❌ Falha ao criar runtime Tokio: {:?}", e);
            panic!("Failed to create Tokio runtime: {:?}", e);
        }
    };
    
    log::info!("🔧 Inicializando AppState...");
    let app_state = match rt.block_on(AppState::new()) {
        Ok(state) => {
            log::info!("✅ AppState inicializado com sucesso");
            state
        },
        Err(e) => {
            log::error!("❌ Falha ao inicializar AppState: {:?}", e);
            panic!("Failed to initialize app state: {:?}", e);
        }
    };

    log::info!("🚀 Iniciando aplicação Tauri...");
    if let Err(e) = tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            login,
            register,
            get_current_user,
            logout,
            get_stats,
            get_documents,
            get_recent_activities
        ])
        .run(tauri::generate_context!())
    {
        log::error!("❌ Erro ao executar aplicação Tauri: {:?}", e);
        panic!("Error while running tauri application: {:?}", e);
    }
}
