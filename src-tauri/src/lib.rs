// AWS 模块
mod database;
mod models;
mod graph_api;
mod browser_automation;
mod commands;
mod aws_sso_client;

// Kiro 模块
mod kiro;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // Initialize database
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = database::init_database(&app_handle).await {
                    eprintln!("Failed to initialize database: {}", e);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // AWS 命令
            commands::get_accounts,
            commands::add_account,
            commands::update_account,
            commands::delete_account,
            commands::delete_all_accounts,
            commands::import_accounts,
            commands::get_settings,
            commands::update_settings,
            commands::start_registration,
            commands::start_batch_registration,
            commands::export_accounts,
            commands::fetch_latest_email,
            commands::kami_login,
            commands::kami_unbind,
            commands::get_notice,
            commands::auto_fetch_kiro_token,
            commands::batch_fetch_kiro_tokens,
            // Kiro 命令
            kiro::kiro::get_kiro_local_token,
            kiro::kiro::get_kiro_telemetry_info,
            kiro::kiro::switch_kiro_account,
            kiro::kiro::reset_kiro_machine_id,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
