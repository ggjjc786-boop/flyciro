mod account;
mod auth;
mod auth_social;
mod aws_sso_client;
mod browser;
mod codewhisperer_client;
mod deep_link_handler;
mod kiro;
mod kiro_auth_client;
mod mcp;
mod powers;
mod process;
mod state;
mod steering;
mod providers;
mod commands;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // Initialize app state
            let app_state = AppState::new();
            app.manage(app_state);

            // Initialize database
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = account::init_database(&app_handle).await {
                    eprintln!("Failed to initialize database: {}", e);
                }
            });

            // Register deep link handler
            deep_link_handler::register_deep_link_handler(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Account commands
            commands::account_cmd::get_accounts,
            commands::account_cmd::add_account,
            commands::account_cmd::update_account,
            commands::account_cmd::delete_account,
            commands::account_cmd::delete_accounts,
            commands::account_cmd::import_accounts,
            commands::account_cmd::export_accounts,
            commands::account_cmd::refresh_account_token,
            commands::account_cmd::refresh_account_usage,
            commands::account_cmd::switch_kiro_account,
            commands::account_cmd::get_kiro_local_token,
            commands::account_cmd::redeem_code,
            // Auth commands
            commands::auth_cmd::get_current_user,
            commands::auth_cmd::logout,
            commands::auth_cmd::desktop_oauth_login,
            commands::auth_cmd::desktop_oauth_callback,
            // Web OAuth commands
            commands::web_oauth_cmd::web_oauth_login,
            commands::web_oauth_cmd::web_oauth_callback,
            // App settings commands
            commands::app_settings_cmd::get_app_settings,
            commands::app_settings_cmd::save_app_settings,
            // Kiro settings commands
            commands::kiro_settings_cmd::get_kiro_settings,
            commands::kiro_settings_cmd::save_kiro_settings,
            commands::kiro_settings_cmd::get_kiro_ide_status,
            commands::kiro_settings_cmd::start_kiro_ide,
            commands::kiro_settings_cmd::stop_kiro_ide,
            // Machine GUID commands
            commands::machine_guid_cmd::get_machine_guid,
            commands::machine_guid_cmd::reset_machine_guid,
            commands::machine_guid_cmd::set_custom_machine_guid,
            commands::machine_guid_cmd::generate_machine_guid,
            commands::machine_guid_cmd::get_bound_machine_id,
            commands::machine_guid_cmd::bind_machine_id_to_account,
            // Proxy commands
            commands::proxy_cmd::get_system_proxy,
            commands::proxy_cmd::test_proxy,
            // MCP commands
            commands::mcp_cmd::get_mcp_servers,
            commands::mcp_cmd::add_mcp_server,
            commands::mcp_cmd::update_mcp_server,
            commands::mcp_cmd::delete_mcp_server,
            commands::mcp_cmd::toggle_mcp_server,
            // Powers commands
            commands::powers_cmd::get_powers_list,
            // Steering commands
            commands::steering_cmd::get_steering_files,
            commands::steering_cmd::read_steering_file,
            commands::steering_cmd::save_steering_file,
            commands::steering_cmd::delete_steering_file,
            // SSO Import commands
            commands::sso_import_cmd::import_sso_tokens,
            // Update commands
            commands::update_cmd::check_for_updates,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
