// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Capturar panics e escrever para stderr
    std::panic::set_hook(Box::new(|panic_info| {
        if let Some(location) = panic_info.location() {
            eprintln!("ARKIVE CRASH: {}:{} - {}", location.file(), location.line(), panic_info);
        } else {
            eprintln!("ARKIVE CRASH: {:?}", panic_info);
        }
    }));
    
    app_lib::run();
}
