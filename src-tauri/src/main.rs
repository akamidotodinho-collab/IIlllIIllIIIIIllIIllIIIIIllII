// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Inicializar logger para debug
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    log::info!("🚀 Iniciando ARKIVE Desktop...");
    
    // Capturar panics e escrever para arquivo de log (Windows compatível)
    std::panic::set_hook(Box::new(|panic_info| {
        log::error!("❌ PANIC: {:?}", panic_info);
        // Não tentar ler stdin no Windows - causa travamento
        if let Some(location) = panic_info.location() {
            eprintln!("ARKIVE CRASH: {}:{} - {}", location.file(), location.line(), panic_info);
        } else {
            eprintln!("ARKIVE CRASH: {:?}", panic_info);
        }
    }));
    
    log::info!("📦 Inicializando aplicação...");
    app_lib::run();
}
