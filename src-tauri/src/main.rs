// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Inicializar logger para debug
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    log::info!("🚀 Iniciando ARKIVE Desktop...");
    
    // Capturar panics para debug - com output visível
    std::panic::set_hook(Box::new(|panic_info| {
        log::error!("❌ PANIC: {:?}", panic_info);
        eprintln!("ARKIVE CRASH: {:?}", panic_info);
        eprintln!("Pressione Enter para fechar...");
        std::io::stdin().read_line(&mut String::new()).ok();
    }));
    
    log::info!("📦 Inicializando aplicação...");
    app_lib::run();
}
