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
            let app_state = AppState::default();
            app.manage(app_state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Account commands
            commands::account_cmd::get_accounts,
            commands::account_cmd::update_account,
            commands::account_cmd::delete_account,
            commands::account_cmd::delete_accounts,
            commands::account_cmd::import_accounts,
            commands::account_cmd::export_accounts,
            commands::account_cmd::refresh_account_token,
            // Auth commands
            commands::auth_cmd::get_current_user,
            commands::auth_cmd::logout,
            // Web OAuth commands
            commands::web_oauth_cmd::web_oauth_login,
            // App settings commands
            commands::app_settings_cmd::get_app_settings,
            commands::app_settings_cmd::save_app_settings,
            // Kiro settings commands
            commands::kiro_settings_cmd::get_kiro_settings,
            // Machine GUID commands
            commands::machine_guid_cmd::set_custom_machine_guid,
            commands::machine_guid_cmd::generate_machine_guid,
            // MCP commands
            commands::mcp_cmd::delete_mcp_server,
            commands::mcp_cmd::toggle_mcp_server,
            // Steering commands
            commands::steering_cmd::get_steering_files,
            commands::steering_cmd::save_steering_file,
            commands::steering_cmd::delete_steering_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
