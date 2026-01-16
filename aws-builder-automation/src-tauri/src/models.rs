use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub email: String,
    pub email_password: String,
    pub client_id: String,
    pub refresh_token: String,
    pub kiro_password: Option<String>,
    pub status: AccountStatus,
    pub error_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    NotRegistered,
    InProgress,
    Registered,
    Error,
}

impl AccountStatus {
    pub fn to_string(&self) -> String {
        match self {
            AccountStatus::NotRegistered => "not_registered".to_string(),
            AccountStatus::InProgress => "in_progress".to_string(),
            AccountStatus::Registered => "registered".to_string(),
            AccountStatus::Error => "error".to_string(),
        }
    }

    pub fn from_string(s: &str) -> Self {
        match s {
            "in_progress" => AccountStatus::InProgress,
            "registered" => AccountStatus::Registered,
            "error" => AccountStatus::Error,
            _ => AccountStatus::NotRegistered,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAccount {
    pub email: String,
    pub email_password: String,
    pub client_id: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUpdate {
    pub id: i64,
    pub email: Option<String>,
    pub email_password: Option<String>,
    pub client_id: Option<String>,
    pub refresh_token: Option<String>,
    pub kiro_password: Option<String>,
    pub status: Option<AccountStatus>,
    pub error_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub browser_mode: BrowserMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BrowserMode {
    Background,
    Foreground,
}

impl BrowserMode {
    pub fn to_string(&self) -> String {
        match self {
            BrowserMode::Background => "background".to_string(),
            BrowserMode::Foreground => "foreground".to_string(),
        }
    }

    pub fn from_string(s: &str) -> Self {
        match s {
            "foreground" => BrowserMode::Foreground,
            _ => BrowserMode::Background,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<ImportError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportError {
    pub line_number: usize,
    pub content: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub id: String,
    pub received_datetime: String,
    pub sent_datetime: String,
    pub subject: String,
    pub body_content: String,
    pub from_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    pub mode: BrowserMode,
    pub os: String,
    pub os_version: String,
    pub device_type: String,
    pub language: String,
    pub window_width: u32,
    pub window_height: u32,
}
