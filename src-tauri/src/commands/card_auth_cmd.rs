use tauri::State;
use crate::card_auth;

#[tauri::command]
pub async fn get_card_notice() -> Result<String, String> {
    card_auth::get_notice()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn verify_card_key(card_key: String) -> Result<String, String> {
    let response = card_auth::card_login(&card_key)
        .await
        .map_err(|e| e.to_string())?;
    
    if response.code == 200 {
        Ok(response.msg)
    } else {
        Err(response.msg)
    }
}

#[tauri::command]
pub async fn unbind_card_key(card_key: String) -> Result<String, String> {
    let response = card_auth::card_unbind(&card_key)
        .await
        .map_err(|e| e.to_string())?;
    
    if response.code == 200 {
        Ok(response.msg)
    } else {
        Err(response.msg)
    }
}

#[tauri::command]
pub fn get_device_code() -> String {
    card_auth::get_machine_code()
}
