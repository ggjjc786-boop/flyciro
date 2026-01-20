use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use md5;
use hex;

const API_URL: &str = "https://zh.xphdfs.me/api.php";
const APP_ID: &str = "10002";
const KEY_MY: &str = "DxhTVxT08L0AD3Dx";
const RC4_KEY: &str = "8HacPHMcsWK10002";

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub code: i32,
    pub msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoticeResponse {
    pub msg: NoticeMsg,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoticeMsg {
    pub app_gg: String,
}

/// RC4 加密/解密
fn rc4(key: &str, data: &[u8]) -> Vec<u8> {
    let key_bytes = key.as_bytes();
    let key_len = key_bytes.len();
    
    // 初始化 S 盒
    let mut s: Vec<u8> = (0..=255).collect();
    let mut j: usize = 0;
    
    for i in 0..256 {
        j = (j + s[i] as usize + key_bytes[i % key_len] as usize) % 256;
        s.swap(i, j);
    }
    
    // 加密/解密
    let mut result = Vec::with_capacity(data.len());
    let mut i: usize = 0;
    let mut j: usize = 0;
    
    for byte in data {
        i = (i + 1) % 256;
        j = (j + s[i] as usize) % 256;
        s.swap(i, j);
        let k = s[(s[i] as usize + s[j] as usize) % 256];
        result.push(byte ^ k);
    }
    
    result
}

/// 字符串转十六进制
fn string_to_hex(s: &str) -> String {
    hex::encode(s.as_bytes())
}

/// 十六进制转字符串
fn hex_to_string(hex_str: &str) -> Result<String> {
    let bytes = hex::decode(hex_str)
        .map_err(|e| anyhow!("Failed to decode hex: {}", e))?;
    String::from_utf8(bytes)
        .map_err(|e| anyhow!("Failed to convert to UTF-8: {}", e))
}

/// RC4 加密（返回十六进制）
fn rc4_encrypt(key: &str, data: &str) -> String {
    let encrypted = rc4(key, data.as_bytes());
    hex::encode(encrypted)
}

/// RC4 解密（从十六进制）
fn rc4_decrypt(key: &str, hex_data: &str) -> Result<String> {
    let bytes = hex::decode(hex_data)
        .map_err(|e| anyhow!("Failed to decode hex: {}", e))?;
    let decrypted = rc4(key, &bytes);
    String::from_utf8(decrypted)
        .map_err(|e| anyhow!("Failed to convert to UTF-8: {}", e))
}

/// 获取时间戳
fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// MD5 哈希
fn md5_hash(data: &str) -> String {
    let digest = md5::compute(data.as_bytes());
    format!("{:x}", digest)
}

/// 获取机器码（使用 Tauri 的 machine ID）
pub fn get_machine_code() -> String {
    // 使用 Tauri 的机器 ID 或生成一个唯一标识
    use crate::kiro::get_machine_id;
    get_machine_id()
}

/// 获取公告
pub async fn get_notice() -> Result<String> {
    let url = format!("{}?api=notice&app={}", API_URL, APP_ID);
    
    let response = reqwest::get(&url).await
        .map_err(|e| anyhow!("Failed to fetch notice: {}", e))?;
    
    let encrypted_data = response.text().await
        .map_err(|e| anyhow!("Failed to read response: {}", e))?;
    
    // 解密响应
    let decrypted = rc4_decrypt(RC4_KEY, &encrypted_data)?;
    
    let notice: NoticeResponse = serde_json::from_str(&decrypted)
        .map_err(|e| anyhow!("Failed to parse notice: {}", e))?;
    
    Ok(notice.msg.app_gg)
}

/// 卡密登录验证
pub async fn card_login(card_key: &str) -> Result<AuthResponse> {
    let machine_code = get_machine_code();
    let timestamp = get_timestamp();
    
    // 生成 Random
    let random_str = format!("{}{}{}", timestamp, KEY_MY, machine_code);
    let random = md5_hash(&random_str);
    
    // 生成签名
    let sign_str = format!("kami={}&markcode={}&t={}&{}", card_key, machine_code, timestamp, KEY_MY);
    let sign = md5_hash(&sign_str);
    
    // 构建请求数据
    let request_data = format!(
        "kami={}&markcode={}&t={}&sign={}&value={}",
        card_key, machine_code, timestamp, sign, random
    );
    
    // RC4 加密请求数据
    let encrypted_request = rc4_encrypt(RC4_KEY, &request_data);
    
    // 发送请求
    let url = format!("{}?api=kmlogon&app={}&data={}", API_URL, APP_ID, encrypted_request);
    
    let response = reqwest::get(&url).await
        .map_err(|e| anyhow!("Failed to send request: {}", e))?;
    
    let encrypted_response = response.text().await
        .map_err(|e| anyhow!("Failed to read response: {}", e))?;
    
    // 解密响应
    let decrypted = rc4_decrypt(RC4_KEY, &encrypted_response)?;
    
    let auth_response: AuthResponse = serde_json::from_str(&decrypted)
        .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
    
    // 验证数据完整性
    if let (Some(time), Some(check)) = (&auth_response.time, &auth_response.check) {
        let verify_str = format!("{}{}{}", time, KEY_MY, random);
        let verify_hash = md5_hash(&verify_str);
        
        if verify_hash != *check {
            return Err(anyhow!("数据被修改，验证失败"));
        }
    }
    
    Ok(auth_response)
}

/// 卡密解绑
pub async fn card_unbind(card_key: &str) -> Result<AuthResponse> {
    let machine_code = get_machine_code();
    let timestamp = get_timestamp();
    
    // 生成 Random
    let random_str = format!("{}{}{}", timestamp, KEY_MY, machine_code);
    let random = md5_hash(&random_str);
    
    // 生成签名
    let sign_str = format!("kami={}&markcode={}&t={}&{}", card_key, machine_code, timestamp, KEY_MY);
    let sign = md5_hash(&sign_str);
    
    // 构建请求数据
    let request_data = format!(
        "kami={}&markcode={}&t={}&sign={}&value={}",
        card_key, machine_code, timestamp, sign, random
    );
    
    // RC4 加密请求数据
    let encrypted_request = rc4_encrypt(RC4_KEY, &request_data);
    
    // 发送请求
    let url = format!("{}?api=kmunmachine&app={}&data={}", API_URL, APP_ID, encrypted_request);
    
    let response = reqwest::get(&url).await
        .map_err(|e| anyhow!("Failed to send request: {}", e))?;
    
    let encrypted_response = response.text().await
        .map_err(|e| anyhow!("Failed to read response: {}", e))?;
    
    // 解密响应
    let decrypted = rc4_decrypt(RC4_KEY, &encrypted_response)?;
    
    let auth_response: AuthResponse = serde_json::from_str(&decrypted)
        .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
    
    Ok(auth_response)
}
