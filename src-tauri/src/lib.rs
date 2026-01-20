// Core modules
mod account;
mod auth;
mod auth_social;
mod state;
mod kiro;
mod kiro_auth_client;
mod codewhisperer_client;
mod providers;
mod mcp;
mod powers;
mod steering;
mod process;
mod browser;
mod deep_link_handler;
mod card_auth;

// Shared modules (used by both main app and auto_register)
mod database;
mod models;
mod graph_api;
mod browser_automation;
mod aws_sso_client;

// Commands
mod commands;

// Auto register feature
mod auto_register;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_deep_link::init())
        .manage(state::AppState::new())
        .setup(|app| {
            // Initialize database
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = database::init_database(&app_handle).await {
                    eprintln!("Failed to initialize database: {}", e);
                }
            });
            
            // Initialize auto_register database
            let app_handle2 = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = auto_register::database::init_database(&app_handle2).await {
                    eprintln!("Failed to initialize auto_register database: {}", e);
                }
            });
            
            // Show main window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::account_cmd::get_accounts,
            commands::account_cmd::delete_account,
            commands::account_cmd::delete_accounts,
            commands::account_cmd::sync_account,
            commands::account_cmd::refresh_account_token,
            commands::account_cmd::verify_account,
            commands::account_cmd::add_account_by_social,
            commands::account_cmd::import_accounts,
            commands::account_cmd::export_accounts,
            commands::account_cmd::add_local_kiro_account,
            commands::account_cmd::add_account_by_idc,
            commands::account_cmd::update_account,
            commands::auth_cmd::get_current_user,
            commands::auth_cmd::logout,
            commands::auth_cmd::kiro_login,
            commands::auth_cmd::handle_kiro_social_callback,
            commands::auth_cmd::add_kiro_account,
            commands::auth_cmd::get_supported_providers,
            commands::app_settings_cmd::get_app_settings,
            commands::app_settings_cmd::save_app_settings,
            commands::app_settings_cmd::bind_machine_id_to_account,
            commands::app_settings_cmd::unbind_machine_id_from_account,
            commands::app_settings_cmd::get_bound_machine_id,
            commands::app_settings_cmd::get_all_bound_machine_ids,
            commands::kiro_settings_cmd::get_kiro_settings,
            commands::kiro_settings_cmd::set_kiro_proxy,
            commands::kiro_settings_cmd::set_kiro_model,
            commands::machine_guid_cmd::get_system_machine_guid,
            commands::machine_guid_cmd::backup_machine_guid,
            commands::machine_guid_cmd::restore_machine_guid,
            commands::machine_guid_cmd::reset_system_machine_guid,
            commands::machine_guid_cmd::get_machine_guid_backup,
            commands::machine_guid_cmd::set_custom_machine_guid,
            commands::machine_guid_cmd::clear_macos_override,
            commands::machine_guid_cmd::generate_machine_guid,
            commands::mcp_cmd::get_mcp_config,
            commands::mcp_cmd::save_mcp_server,
            commands::mcp_cmd::delete_mcp_server,
            commands::mcp_cmd::toggle_mcp_server,
            commands::powers_cmd::get_installed_powers,
            commands::powers_cmd::get_all_powers,
            commands::powers_cmd::get_powers_registry,
            commands::powers_cmd::install_power,
            commands::powers_cmd::uninstall_power,
            commands::proxy_cmd::detect_system_proxy,
            commands::web_oauth_cmd::web_oauth_initiate,
            commands::web_oauth_cmd::web_oauth_complete,
            commands::web_oauth_cmd::web_oauth_refresh,
            commands::web_oauth_cmd::web_oauth_login,
            commands::web_oauth_cmd::web_oauth_close_window,
            commands::auto_register_cmd::auto_register_get_accounts,
            commands::auto_register_cmd::auto_register_add_account,
            commands::auto_register_cmd::auto_register_update_account,
            commands::auto_register_cmd::auto_register_delete_account,
            commands::auto_register_cmd::auto_register_delete_all_accounts,
            commands::auto_register_cmd::auto_register_import_accounts,
            commands::auto_register_cmd::auto_register_get_settings,
            commands::auto_register_cmd::auto_register_update_settings,
            commands::auto_register_cmd::auto_register_start_registration,
            commands::auto_register_cmd::auto_register_start_batch_registration,
            commands::auto_register_cmd::auto_register_export_accounts,
            commands::auto_register_cmd::auto_register_fetch_latest_email,
            commands::auto_register_cmd::auto_register_get_kiro_credentials,
            commands::auto_register_cmd::auto_register_batch_fetch_kiro_credentials,
            commands::auto_register_cmd::auto_register_import_to_main,
            commands::auto_register_cmd::auto_register_get_credentials_and_import,
            commands::card_auth_cmd::get_card_notice,
            commands::card_auth_cmd::verify_card_key,
            commands::card_auth_cmd::unbind_card_key,
            commands::card_auth_cmd::get_device_code,
            crate::kiro::switch_kiro_account,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
