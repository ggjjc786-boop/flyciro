use crate::auto_register::models::*;
use rusqlite::{Connection, params};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use chrono::Utc;
use anyhow::Result;

pub struct DbState(pub Mutex<Connection>);

pub async fn init_database(app: &AppHandle) -> Result<()> {
    let app_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_dir)?;

    let db_path = app_dir.join("database.db");
    let conn = Connection::open(&db_path)?;

    // Create accounts table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS accounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT NOT NULL UNIQUE,
            email_password TEXT NOT NULL,
            client_id TEXT NOT NULL,
            refresh_token TEXT NOT NULL,
            kiro_password TEXT,
            status TEXT NOT NULL DEFAULT 'not_registered',
            error_reason TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            kiro_client_id TEXT,
            kiro_client_secret TEXT,
            kiro_refresh_token TEXT,
            kiro_access_token TEXT,
            kiro_id_token TEXT
        )",
        [],
    )?;

    // Add kiro columns if they don't exist (for existing databases)
    let _ = conn.execute("ALTER TABLE accounts ADD COLUMN kiro_client_id TEXT", []);
    let _ = conn.execute("ALTER TABLE accounts ADD COLUMN kiro_client_secret TEXT", []);
    let _ = conn.execute("ALTER TABLE accounts ADD COLUMN kiro_refresh_token TEXT", []);
    let _ = conn.execute("ALTER TABLE accounts ADD COLUMN kiro_access_token TEXT", []);
    let _ = conn.execute("ALTER TABLE accounts ADD COLUMN kiro_id_token TEXT", []);

    // Create settings table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            browser_mode TEXT NOT NULL DEFAULT 'background'
        )",
        [],
    )?;

    // Insert default settings if not exists
    conn.execute(
        "INSERT OR IGNORE INTO settings (id, browser_mode) VALUES (1, 'background')",
        [],
    )?;

    // Create indexes for better performance
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_accounts_status ON accounts(status)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_accounts_email ON accounts(email)",
        [],
    )?;

    app.manage(DbState(Mutex::new(conn)));

    Ok(())
}

pub fn get_all_accounts(conn: &Connection) -> Result<Vec<Account>> {
    let mut stmt = conn.prepare(
        "SELECT id, email, email_password, client_id, refresh_token, kiro_password,
         status, error_reason, created_at, updated_at,
         kiro_client_id, kiro_client_secret, kiro_refresh_token, kiro_access_token, kiro_id_token
         FROM accounts
         ORDER BY created_at DESC"
    )?;

    let accounts = stmt.query_map([], |row| {
        Ok(Account {
            id: row.get(0)?,
            email: row.get(1)?,
            email_password: row.get(2)?,
            client_id: row.get(3)?,
            refresh_token: row.get(4)?,
            kiro_password: row.get(5)?,
            status: AccountStatus::from_string(&row.get::<_, String>(6)?),
            error_reason: row.get(7)?,
            created_at: row.get::<_, String>(8)?.parse().unwrap_or(Utc::now()),
            updated_at: row.get::<_, String>(9)?.parse().unwrap_or(Utc::now()),
            kiro_client_id: row.get(10)?,
            kiro_client_secret: row.get(11)?,
            kiro_refresh_token: row.get(12)?,
            kiro_access_token: row.get(13)?,
            kiro_id_token: row.get(14)?,
        })
    })?
    .collect::<rusqlite::Result<Vec<Account>>>()?;

    Ok(accounts)
}

pub fn get_accounts_by_status(conn: &Connection, status: &str) -> Result<Vec<Account>> {
    let mut stmt = conn.prepare(
        "SELECT id, email, email_password, client_id, refresh_token, kiro_password,
         status, error_reason, created_at, updated_at,
         kiro_client_id, kiro_client_secret, kiro_refresh_token, kiro_access_token, kiro_id_token
         FROM accounts
         WHERE status = ?1
         ORDER BY created_at DESC"
    )?;

    let accounts = stmt.query_map([status], |row| {
        Ok(Account {
            id: row.get(0)?,
            email: row.get(1)?,
            email_password: row.get(2)?,
            client_id: row.get(3)?,
            refresh_token: row.get(4)?,
            kiro_password: row.get(5)?,
            status: AccountStatus::from_string(&row.get::<_, String>(6)?),
            error_reason: row.get(7)?,
            created_at: row.get::<_, String>(8)?.parse().unwrap_or(Utc::now()),
            updated_at: row.get::<_, String>(9)?.parse().unwrap_or(Utc::now()),
            kiro_client_id: row.get(10)?,
            kiro_client_secret: row.get(11)?,
            kiro_refresh_token: row.get(12)?,
            kiro_access_token: row.get(13)?,
            kiro_id_token: row.get(14)?,
        })
    })?
    .collect::<rusqlite::Result<Vec<Account>>>()?;

    Ok(accounts)
}

pub fn insert_account(conn: &Connection, account: NewAccount) -> Result<i64> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO accounts (email, email_password, client_id, refresh_token, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            account.email,
            account.email_password,
            account.client_id,
            account.refresh_token,
            AccountStatus::NotRegistered.to_string(),
            now,
            now
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn update_account(conn: &Connection, update: AccountUpdate) -> Result<()> {
    let now = Utc::now().to_rfc3339();

    let mut query = String::from("UPDATE accounts SET updated_at = ?1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now)];
    let mut param_index = 2;

    if let Some(email) = update.email {
        query.push_str(&format!(", email = ?{}", param_index));
        params.push(Box::new(email));
        param_index += 1;
    }

    if let Some(email_password) = update.email_password {
        query.push_str(&format!(", email_password = ?{}", param_index));
        params.push(Box::new(email_password));
        param_index += 1;
    }

    if let Some(client_id) = update.client_id {
        query.push_str(&format!(", client_id = ?{}", param_index));
        params.push(Box::new(client_id));
        param_index += 1;
    }

    if let Some(refresh_token) = update.refresh_token {
        query.push_str(&format!(", refresh_token = ?{}", param_index));
        params.push(Box::new(refresh_token));
        param_index += 1;
    }

    if let Some(kiro_password) = update.kiro_password {
        query.push_str(&format!(", kiro_password = ?{}", param_index));
        params.push(Box::new(kiro_password));
        param_index += 1;
    }

    if let Some(status) = update.status {
        query.push_str(&format!(", status = ?{}", param_index));
        params.push(Box::new(status.to_string()));
        param_index += 1;
    }

    if let Some(error_reason) = update.error_reason {
        query.push_str(&format!(", error_reason = ?{}", param_index));
        params.push(Box::new(error_reason));
        param_index += 1;
    }

    if let Some(kiro_client_id) = update.kiro_client_id {
        query.push_str(&format!(", kiro_client_id = ?{}", param_index));
        params.push(Box::new(kiro_client_id));
        param_index += 1;
    }

    if let Some(kiro_client_secret) = update.kiro_client_secret {
        query.push_str(&format!(", kiro_client_secret = ?{}", param_index));
        params.push(Box::new(kiro_client_secret));
        param_index += 1;
    }

    if let Some(kiro_refresh_token) = update.kiro_refresh_token {
        query.push_str(&format!(", kiro_refresh_token = ?{}", param_index));
        params.push(Box::new(kiro_refresh_token));
        param_index += 1;
    }

    if let Some(kiro_access_token) = update.kiro_access_token {
        query.push_str(&format!(", kiro_access_token = ?{}", param_index));
        params.push(Box::new(kiro_access_token));
        param_index += 1;
    }

    if let Some(kiro_id_token) = update.kiro_id_token {
        query.push_str(&format!(", kiro_id_token = ?{}", param_index));
        params.push(Box::new(kiro_id_token));
        param_index += 1;
    }

    query.push_str(&format!(" WHERE id = ?{}", param_index));
    params.push(Box::new(update.id));

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    conn.execute(&query, params_refs.as_slice())?;

    Ok(())
}

pub fn delete_account(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM accounts WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn delete_all_accounts(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM accounts", [])?;
    Ok(())
}

pub fn get_settings(conn: &Connection) -> Result<Settings> {
    let browser_mode: String = conn.query_row(
        "SELECT browser_mode FROM settings WHERE id = 1",
        [],
        |row| row.get(0),
    )?;

    Ok(Settings {
        browser_mode: BrowserMode::from_string(&browser_mode),
    })
}

pub fn update_settings(conn: &Connection, settings: Settings) -> Result<()> {
    conn.execute(
        "UPDATE settings SET browser_mode = ?1 WHERE id = 1",
        params![settings.browser_mode.to_string()],
    )?;
    Ok(())
}

pub fn get_account_by_id(conn: &Connection, id: i64) -> Result<Account> {
    let account = conn.query_row(
        "SELECT id, email, email_password, client_id, refresh_token, kiro_password,
         status, error_reason, created_at, updated_at,
         kiro_client_id, kiro_client_secret, kiro_refresh_token, kiro_access_token, kiro_id_token
         FROM accounts
         WHERE id = ?1",
        params![id],
        |row| {
            Ok(Account {
                id: row.get(0)?,
                email: row.get(1)?,
                email_password: row.get(2)?,
                client_id: row.get(3)?,
                refresh_token: row.get(4)?,
                kiro_password: row.get(5)?,
                status: AccountStatus::from_string(&row.get::<_, String>(6)?),
                error_reason: row.get(7)?,
                created_at: row.get::<_, String>(8)?.parse().unwrap_or(Utc::now()),
                updated_at: row.get::<_, String>(9)?.parse().unwrap_or(Utc::now()),
                kiro_client_id: row.get(10)?,
                kiro_client_secret: row.get(11)?,
                kiro_refresh_token: row.get(12)?,
                kiro_access_token: row.get(13)?,
                kiro_id_token: row.get(14)?,
            })
        },
    )?;

    Ok(account)
}
