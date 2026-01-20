use serde::{Deserialize, Serialize};
use crate::card_auth;

#[derive(Debug, Serialize, Deserialize)]
pub struct CardAuthResult {
    pub message: String,
    pub expire_time: Option<String>,
}

#[tauri::command]
pub async fn get_card_notice() -> Result<String, String> {
    card_auth::get_notice()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn verify_card_key(card_key: String) -> Result<CardAuthResult, String> {
    let (code, msg, expire_time) = card_auth::card_login(&card_key)
        .await
        .map_err(|e| e.to_string())?;
    
    if code == 200 {
        Ok(CardAuthResult {
            message: msg,
            expire_time,
        })
    } else {
        Err(msg)
    }
}

#[tauri::command]
pub async fn unbind_card_key(card_key: String) -> Result<String, String> {
    let (code, msg) = card_auth::card_unbind(&card_key)
        .await
        .map_err(|e| e.to_string())?;
    
    if code == 200 {
        Ok(msg)
    } else {
        Err(msg)
    }
}

#[tauri::command]
pub fn get_device_code() -> String {
    card_auth::get_machine_code()
}
