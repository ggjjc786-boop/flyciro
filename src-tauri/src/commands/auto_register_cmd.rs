use crate::auto_register::database::{self, DbState};
use crate::auto_register::models::*;
use crate::auto_register::graph_api::GraphApiClient;
use crate::auto_register::browser_automation::BrowserAutomation;
use crate::auto_register::aws_sso_client::AWSSSOClient;
use tauri::State;
use anyhow::{Result, Context, anyhow};

#[tauri::command]
pub async fn auto_register_get_accounts(
    db: State<'_, DbState>,
    status_filter: Option<String>,
) -> Result<Vec<Account>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let accounts = if let Some(status) = status_filter {
        database::get_accounts_by_status(&conn, &status)
    } else {
        database::get_all_accounts(&conn)
    };

    accounts.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn auto_register_add_account(
    db: State<'_, DbState>,
    account: NewAccount,
) -> Result<i64, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::insert_account(&conn, account).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn auto_register_update_account(
    db: State<'_, DbState>,
    update: AccountUpdate,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::update_account(&conn, update).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn auto_register_delete_account(
    db: State<'_, DbState>,
    id: i64,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::delete_account(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn auto_register_delete_all_accounts(
    db: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::delete_all_accounts(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn auto_register_import_accounts(
    db: State<'_, DbState>,
    content: String,
) -> Result<ImportResult, String> {
    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors = Vec::new();

    let lines: Vec<&str> = content.lines().collect();

    for (index, line) in lines.iter().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split("----").collect();

        if parts.len() != 4 {
            error_count += 1;
            errors.push(ImportError {
                line_number: index + 1,
                content: line.to_string(),
                reason: format!("Invalid format: expected 4 fields separated by '----', got {}", parts.len()),
            });
            continue;
        }

        let email = parts[0].trim();
        let password = parts[1].trim();
        let client_id = parts[2].trim();
        let refresh_token = parts[3].trim();

        if !email.contains('@') {
            error_count += 1;
            errors.push(ImportError {
                line_number: index + 1,
                content: line.to_string(),
                reason: "Invalid email address".to_string(),
            });
            continue;
        }

        if email.is_empty() || password.is_empty() || client_id.is_empty() || refresh_token.is_empty() {
            error_count += 1;
            errors.push(ImportError {
                line_number: index + 1,
                content: line.to_string(),
                reason: "One or more fields are empty".to_string(),
            });
            continue;
        }

        let new_account = NewAccount {
            email: email.to_string(),
            email_password: password.to_string(),
            client_id: client_id.to_string(),
            refresh_token: refresh_token.to_string(),
        };

        let conn = db.0.lock().map_err(|e| e.to_string())?;
        match database::insert_account(&conn, new_account) {
            Ok(_) => success_count += 1,
            Err(e) => {
                error_count += 1;
                errors.push(ImportError {
                    line_number: index + 1,
                    content: line.to_string(),
                    reason: format!("Database error: {}", e),
                });
            }
        }
    }

    Ok(ImportResult {
        success_count,
        error_count,
        errors,
    })
}

#[tauri::command]
pub async fn auto_register_get_settings(
    db: State<'_, DbState>,
) -> Result<Settings, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::get_settings(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn auto_register_update_settings(
    db: State<'_, DbState>,
    settings: Settings,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::update_settings(&conn, settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn auto_register_start_registration(
    db: State<'_, DbState>,
    account_id: i64,
) -> Result<String, String> {
    let account = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_account_by_id(&conn, account_id).map_err(|e| e.to_string())?
    };

    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::update_account(
            &conn,
            AccountUpdate {
                id: account_id,
                email: None,
                email_password: None,
                client_id: None,
                refresh_token: None,
                kiro_password: None,
                status: Some(AccountStatus::InProgress),
                error_reason: None,
                kiro_client_id: None,
                kiro_client_secret: None,
                kiro_refresh_token: None,
                kiro_access_token: None,
                kiro_id_token: None,
            },
        )
        .map_err(|e| e.to_string())?;
    }

    let settings = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_settings(&conn).map_err(|e| e.to_string())?
    };

    let names = vec![
        "Zhang Wei", "Wang Fang", "Li Na", "Liu Yang", "Chen Jing",
        "Zhang Min", "Wang Lei", "Li Qiang", "Liu Min", "Chen Wei",
    ];
    let random_name = names[rand::random::<usize>() % names.len()];

    let result = perform_registration(
        &account.email,
        &account.email_password,
        &account.client_id,
        &account.refresh_token,
        random_name,
        settings.browser_mode,
    ).await;

    match result {
        Ok(kiro_password) => {
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            database::update_account(
                &conn,
                AccountUpdate {
                    id: account_id,
                    email: None,
                    email_password: None,
                    client_id: None,
                    refresh_token: None,
                    kiro_password: Some(kiro_password.clone()),
                    status: Some(AccountStatus::Registered),
                    error_reason: None,
                    kiro_client_id: None,
                    kiro_client_secret: None,
                    kiro_refresh_token: None,
                    kiro_access_token: None,
                    kiro_id_token: None,
                },
            )
            .map_err(|e| e.to_string())?;

            Ok(format!("Registration completed successfully. Password: {}", kiro_password))
        }
        Err(e) => {
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            database::update_account(
                &conn,
                AccountUpdate {
                    id: account_id,
                    email: None,
                    email_password: None,
                    client_id: None,
                    refresh_token: None,
                    kiro_password: None,
                    status: Some(AccountStatus::Error),
                    error_reason: Some(e.to_string()),
                    kiro_client_id: None,
                    kiro_client_secret: None,
                    kiro_refresh_token: None,
                    kiro_access_token: None,
                    kiro_id_token: None,
                },
            )
            .map_err(|e| e.to_string())?;

            Err(e.to_string())
        }
    }
}

async fn perform_registration(
    email: &str,
    _email_password: &str,
    client_id: &str,
    refresh_token: &str,
    name: &str,
    browser_mode: BrowserMode,
) -> Result<String> {
    let (width, height) = BrowserAutomation::generate_random_window_size();
    let os_version = BrowserAutomation::generate_random_os_version();

    let config = BrowserConfig {
        mode: browser_mode,
        os: "Windows".to_string(),
        os_version,
        device_type: "PC".to_string(),
        language: "zh-CN".to_string(),
        window_width: width,
        window_height: height,
    };

    let automation = BrowserAutomation::new(config);
    let browser = automation.launch_browser()?;
    let tab = browser.new_tab().context("Failed to create new tab")?;

    automation.apply_fingerprint_protection(&tab)?;

    tab.navigate_to("https://app.kiro.dev/signin")
        .context("Failed to navigate to signin page")?;
    tab.wait_until_navigated()?;

    std::thread::sleep(std::time::Duration::from_secs(3));

    let google_button_xpath = "/html/body/div[2]/div/div[1]/main/div/div/div/div/div/div/div/div[1]/button[3]";

    if automation.wait_for_element(&tab, google_button_xpath, 10).await? {
        automation.click_element(&tab, google_button_xpath)?;
        std::thread::sleep(std::time::Duration::from_secs(4));
    } else {
        return Err(anyhow!("Google sign-in button not found"));
    }

    let email_page_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div";

    if automation.wait_for_element(&tab, email_page_xpath, 15).await? {
        std::thread::sleep(std::time::Duration::from_millis(500));

        let email_input_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div/div[2]/div/div[2]/div/div/div/div/div/input";
        automation.input_text(&tab, email_input_xpath, email)?;
        std::thread::sleep(std::time::Duration::from_millis(2000));

        let continue_button_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div/div[3]/button";
        automation.click_element(&tab, continue_button_xpath)?;
        std::thread::sleep(std::time::Duration::from_secs(4));
    } else {
        return Err(anyhow!("Email input page not found"));
    }

    let name_page_xpath = "/html/body/div[2]/div/div/div[2]/div/div/div/div[2]/div";

    if automation.wait_for_element(&tab, name_page_xpath, 15).await? {
        std::thread::sleep(std::time::Duration::from_millis(500));

        let name_input_xpath = "/html/body/div[2]/div/div/div[1]/div/div/form/fieldset/div/div/div/div/div/div/div/div/div[3]/div/div[2]/div/div/div/div/div/input";
        automation.input_text(&tab, name_input_xpath, name)?;
        std::thread::sleep(std::time::Duration::from_millis(2000));

        let continue_button_xpath = "/html/body/div[2]/div/div/div[1]/div/div/form/fieldset/div/div/div/div/div/div/div/div/div[4]/button";
        automation.click_element(&tab, continue_button_xpath)?;
        std::thread::sleep(std::time::Duration::from_secs(4));
    } else {
        return Err(anyhow!("Name input page not found"));
    }

    let verification_page_xpath = "/html/body/div[2]/div/div/div[1]/div/div/div[2]/form/fieldset/div/div/div/div/div/div";

    if automation.wait_for_element(&tab, verification_page_xpath, 15).await? {
        std::thread::sleep(std::time::Duration::from_millis(500));

        let graph_client = GraphApiClient::new();

        let verification_code = match graph_client
            .wait_for_verification_code(client_id, refresh_token, email, 60)
            .await
        {
            Ok(code) => code,
            Err(_) => {
                let resend_button_xpath = "/html/body/div[2]/div/div/div[1]/div/div/div[2]/form/fieldset/div/div/div/div/div/div/div[3]/div/div[2]/div/div/div/div/div/div[1]/div/div[2]/button";
                automation.click_element(&tab, resend_button_xpath)?;
                std::thread::sleep(std::time::Duration::from_secs(5));

                graph_client
                    .wait_for_verification_code(client_id, refresh_token, email, 60)
                    .await?
            }
        };

        let code_input_xpath = "/html/body/div[2]/div/div/div[1]/div/div/div[2]/form/fieldset/div/div/div/div/div/div/div[3]/div/div[2]/div/div/div/div/div/div[1]/div/div[1]/div/input";
        automation.input_text(&tab, code_input_xpath, &verification_code)?;
        std::thread::sleep(std::time::Duration::from_millis(2000));

        let continue_button_xpath = "/html/body/div[2]/div/div/div[1]/div/div/div[2]/form/fieldset/div/div/div/div/div/div/div[4]/button";
        automation.click_element(&tab, continue_button_xpath)?;
        std::thread::sleep(std::time::Duration::from_secs(4));
    } else {
        return Err(anyhow!("Verification code page not found"));
    }

    let password_page_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div[2]/div[3]/button";

    if automation.wait_for_element(&tab, password_page_xpath, 15).await? {
        std::thread::sleep(std::time::Duration::from_millis(500));

        let password = generate_secure_password();

        let password_input_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div[1]/div[3]/div/div[2]/div/div/div/div/div/span/span/div/input";
        automation.input_text(&tab, password_input_xpath, &password)?;
        std::thread::sleep(std::time::Duration::from_millis(2000));

        let confirm_password_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div[1]/div[4]/div/div[2]/div/div/div/div/div/input";
        automation.input_text(&tab, confirm_password_xpath, &password)?;
        std::thread::sleep(std::time::Duration::from_millis(2000));

        let continue_button_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div[2]/div[3]/button";
        automation.click_element(&tab, continue_button_xpath)?;
        std::thread::sleep(std::time::Duration::from_secs(4));

        let success_page_xpath = "/html/body/div[2]/div/div[1]/main/div/div[1]/div/div/div/div/div[2]";

        if automation.wait_for_element(&tab, success_page_xpath, 15).await? {
            automation.clear_browser_data()?;
            return Ok(password);
        } else {
            return Err(anyhow!("Success page not found - registration may have failed"));
        }
    } else {
        return Err(anyhow!("Password input page not found"));
    }
}

fn generate_secure_password() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
    let mut rng = rand::thread_rng();

    let password: String = (0..16)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    password
}

#[tauri::command]
pub async fn auto_register_start_batch_registration(
    db: State<'_, DbState>,
) -> Result<String, String> {
    let accounts = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_accounts_by_status(&conn, "not_registered").map_err(|e| e.to_string())?
    };

    if accounts.is_empty() {
        return Ok("没有需要注册的账号".to_string());
    }

    let total_count = accounts.len();
    let mut success_count = 0;
    let mut error_count = 0;

    let settings = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_settings(&conn).map_err(|e| e.to_string())?
    };

    for account in accounts {
        {
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            database::update_account(
                &conn,
                AccountUpdate {
                    id: account.id,
                    email: None,
                    email_password: None,
                    client_id: None,
                    refresh_token: None,
                    kiro_password: None,
                    status: Some(AccountStatus::InProgress),
                    error_reason: None,
                    kiro_client_id: None,
                    kiro_client_secret: None,
                    kiro_refresh_token: None,
                    kiro_access_token: None,
                    kiro_id_token: None,
                },
            )
            .map_err(|e| e.to_string())?;
        }

        let names = vec![
            "Zhang Wei", "Wang Fang", "Li Na", "Liu Yang", "Chen Jing",
            "Zhang Min", "Wang Lei", "Li Qiang", "Liu Min", "Chen Wei",
        ];
        let random_name = names[rand::random::<usize>() % names.len()];

        let result = perform_registration(
            &account.email,
            &account.email_password,
            &account.client_id,
            &account.refresh_token,
            random_name,
            settings.browser_mode.clone(),
        ).await;

        match result {
            Ok(kiro_password) => {
                let conn = db.0.lock().map_err(|e| e.to_string())?;
                database::update_account(
                    &conn,
                    AccountUpdate {
                        id: account.id,
                        email: None,
                        email_password: None,
                        client_id: None,
                        refresh_token: None,
                        kiro_password: Some(kiro_password),
                        status: Some(AccountStatus::Registered),
                        error_reason: None,
                        kiro_client_id: None,
                        kiro_client_secret: None,
                        kiro_refresh_token: None,
                        kiro_access_token: None,
                        kiro_id_token: None,
                    },
                )
                .map_err(|e| e.to_string())?;

                success_count += 1;
            }
            Err(e) => {
                let conn = db.0.lock().map_err(|e| e.to_string())?;
                database::update_account(
                    &conn,
                    AccountUpdate {
                        id: account.id,
                        email: None,
                        email_password: None,
                        client_id: None,
                        refresh_token: None,
                        kiro_password: None,
                        status: Some(AccountStatus::Error),
                        error_reason: Some(e.to_string()),
                        kiro_client_id: None,
                        kiro_client_secret: None,
                        kiro_refresh_token: None,
                        kiro_access_token: None,
                        kiro_id_token: None,
                    },
                )
                .map_err(|e| e.to_string())?;

                error_count += 1;
            }
        }
    }

    Ok(format!(
        "批量注册完成！总计: {}, 成功: {}, 失败: {}",
        total_count, success_count, error_count
    ))
}

#[tauri::command]
pub async fn auto_register_export_accounts(
    db: State<'_, DbState>,
    status_filter: Option<String>,
) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let accounts = if let Some(status) = status_filter {
        database::get_accounts_by_status(&conn, &status)
    } else {
        database::get_all_accounts(&conn)
    };

    let accounts = accounts.map_err(|e| e.to_string())?;

    if accounts.is_empty() {
        return Ok(String::new());
    }

    let mut lines = Vec::new();
    for account in accounts {
        let kiro_pwd = account.kiro_password.as_deref().unwrap_or("");
        let status_str = match account.status {
            AccountStatus::NotRegistered => "not_registered",
            AccountStatus::InProgress => "in_progress",
            AccountStatus::Registered => "registered",
            AccountStatus::Error => "error",
        };
        let line = format!(
            "{}----{}----{}----{}----{}----{}",
            account.email,
            account.email_password,
            account.client_id,
            account.refresh_token,
            kiro_pwd,
            status_str
        );
        lines.push(line);
    }

    Ok(lines.join("\n"))
}

#[tauri::command]
pub async fn auto_register_fetch_latest_email(
    db: State<'_, DbState>,
    account_id: i64,
) -> Result<Vec<EmailMessage>, String> {
    let account = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_account_by_id(&conn, account_id).map_err(|e| e.to_string())?
    };
    
    let graph_client = GraphApiClient::new();
    
    let access_token = graph_client
        .get_access_token(&account.client_id, &account.refresh_token)
        .await
        .map_err(|e| format!("获取访问令牌失败: {}", e))?;
    
    let emails = graph_client
        .fetch_recent_emails(&access_token, &account.email, 10)
        .await
        .map_err(|e| format!("获取邮件失败: {}", e))?;
    
    Ok(emails)
}

#[tauri::command]
pub async fn get_kiro_credentials(
    db: State<'_, DbState>,
    account_id: i64,
) -> Result<String, String> {
    let account = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_account_by_id(&conn, account_id).map_err(|e| e.to_string())?
    };

    if account.status != AccountStatus::Registered {
        return Err("账号尚未完成注册，请先完成注册".to_string());
    }

    let settings = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_settings(&conn).map_err(|e| e.to_string())?
    };

    let result = perform_kiro_login(
        &account.email,
        account.kiro_password.as_deref().unwrap_or(""),
        &account.client_id,
        &account.refresh_token,
        settings.browser_mode,
    ).await;

    match result {
        Ok(credentials) => {
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            database::update_account(
                &conn,
                AccountUpdate {
                    id: account_id,
                    email: None,
                    email_password: None,
                    client_id: None,
                    refresh_token: None,
                    kiro_password: None,
                    status: None,
                    error_reason: None,
                    kiro_client_id: Some(credentials.client_id.clone()),
                    kiro_client_secret: Some(credentials.client_secret.clone()),
                    kiro_refresh_token: Some(credentials.refresh_token.clone()),
                    kiro_access_token: Some(credentials.access_token.clone()),
                    kiro_id_token: credentials.id_token.clone(),
                },
            )
            .map_err(|e| e.to_string())?;

            Ok(format!("成功获取 Kiro 凭证！Client ID: {}", credentials.client_id))
        }
        Err(e) => {
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn batch_fetch_kiro_credentials(
    db: State<'_, DbState>,
) -> Result<String, String> {
    let accounts = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_accounts_by_status(&conn, "registered").map_err(|e| e.to_string())?
    };

    let accounts_without_credentials: Vec<_> = accounts
        .into_iter()
        .filter(|a| a.kiro_client_id.is_none())
        .collect();

    if accounts_without_credentials.is_empty() {
        return Ok("没有需要获取凭证的账号".to_string());
    }

    let total_count = accounts_without_credentials.len();
    let mut success_count = 0;
    let mut error_count = 0;

    let settings = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_settings(&conn).map_err(|e| e.to_string())?
    };

    for account in accounts_without_credentials {
        let result = perform_kiro_login(
            &account.email,
            account.kiro_password.as_deref().unwrap_or(""),
            &account.client_id,
            &account.refresh_token,
            settings.browser_mode.clone(),
        ).await;

        match result {
            Ok(credentials) => {
                let conn = db.0.lock().map_err(|e| e.to_string())?;
                database::update_account(
                    &conn,
                    AccountUpdate {
                        id: account.id,
                        email: None,
                        email_password: None,
                        client_id: None,
                        refresh_token: None,
                        kiro_password: None,
                        status: None,
                        error_reason: None,
                        kiro_client_id: Some(credentials.client_id),
                        kiro_client_secret: Some(credentials.client_secret),
                        kiro_refresh_token: Some(credentials.refresh_token),
                        kiro_access_token: Some(credentials.access_token),
                        kiro_id_token: credentials.id_token,
                    },
                )
                .map_err(|e| e.to_string())?;

                success_count += 1;
            }
            Err(_) => {
                error_count += 1;
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    Ok(format!(
        "批量获取凭证完成！总计: {}, 成功: {}, 失败: {}",
        total_count, success_count, error_count
    ))
}

async fn perform_kiro_login(
    email: &str,
    kiro_password: &str,
    email_client_id: &str,
    email_refresh_token: &str,
    browser_mode: BrowserMode,
) -> Result<KiroCredentials> {
    let start_url = "https://view.awsapps.com/start";
    let sso_client = AWSSSOClient::new("us-east-1");

    println!("[Kiro Login] Step 1: Registering device client...");
    let client_reg = sso_client.register_device_client(start_url).await
        .map_err(|e| anyhow!("注册设备客户端失败: {}", e))?;

    println!("[Kiro Login] Step 2: Starting device authorization...");
    let device_auth = sso_client.start_device_authorization(
        &client_reg.client_id,
        &client_reg.client_secret,
        start_url,
    ).await
        .map_err(|e| anyhow!("发起设备授权失败: {}", e))?;

    println!("[Kiro Login] Step 3: Launching browser for authorization...");
    let verification_url = device_auth.verification_uri_complete.as_ref()
        .unwrap_or(&device_auth.verification_uri);
    
    let browser_result = perform_browser_authorization(
        verification_url,
        email,
        kiro_password,
        email_client_id,
        email_refresh_token,
        browser_mode,
    ).await;

    if let Err(e) = browser_result {
        return Err(anyhow!("浏览器授权失败: {}", e));
    }

    println!("[Kiro Login] Step 4: Polling for token...");
    let poll_interval = device_auth.interval.unwrap_or(5) as u64;
    let max_attempts = (device_auth.expires_in / poll_interval as i64) as usize;

    for attempt in 0..max_attempts {
        std::thread::sleep(std::time::Duration::from_secs(poll_interval));

        match sso_client.poll_device_token(
            &client_reg.client_id,
            &client_reg.client_secret,
            &device_auth.device_code,
        ).await {
            Ok(DevicePollResult::Success(token)) => {
                println!("[Kiro Login] Token obtained successfully!");
                return Ok(KiroCredentials {
                    client_id: client_reg.client_id,
                    client_secret: client_reg.client_secret,
                    refresh_token: token.refresh_token,
                    access_token: token.access_token,
                    id_token: token.id_token,
                });
            }
            Ok(DevicePollResult::Pending) => {
                println!("[Kiro Login] Authorization pending... (attempt {}/{})", attempt + 1, max_attempts);
                continue;
            }
            Ok(DevicePollResult::SlowDown) => {
                println!("[Kiro Login] Rate limited, slowing down...");
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
            Ok(DevicePollResult::Expired) => {
                return Err(anyhow!("设备授权已过期"));
            }
            Ok(DevicePollResult::Denied) => {
                return Err(anyhow!("授权被拒绝"));
            }
            Err(e) => {
                return Err(anyhow!("轮询 Token 失败: {}", e));
            }
        }
    }

    Err(anyhow!("获取 Token 超时"))
}

async fn perform_browser_authorization(
    verification_url: &str,
    email: &str,
    kiro_password: &str,
    email_client_id: &str,
    email_refresh_token: &str,
    browser_mode: BrowserMode,
) -> Result<()> {
    let (width, height) = BrowserAutomation::generate_random_window_size();
    let os_version = BrowserAutomation::generate_random_os_version();

    let config = BrowserConfig {
        mode: browser_mode,
        os: "Windows".to_string(),
        os_version,
        device_type: "PC".to_string(),
        language: "zh-CN".to_string(),
        window_width: width,
        window_height: height,
    };

    let automation = BrowserAutomation::new(config);
    let browser = automation.launch_browser()?;
    let tab = browser.new_tab().context("Failed to create new tab")?;

    automation.apply_fingerprint_protection(&tab)?;

    println!("[Browser Auth] Navigating to: {}", verification_url);
    tab.navigate_to(verification_url)
        .context("Failed to navigate to verification URL")?;
    tab.wait_until_navigated()?;

    std::thread::sleep(std::time::Duration::from_secs(3));

    println!("[Browser Auth] Looking for confirm button...");
    let confirm_button_selectors = vec![
        "//button[contains(text(), 'Confirm and continue')]",
        "//button[contains(text(), 'Confirm')]",
        "//input[@type='submit' and contains(@value, 'Confirm')]",
        "//*[@id='cli_verification_btn']",
    ];
    
    for selector in &confirm_button_selectors {
        if automation.wait_for_element(&tab, selector, 3).await.unwrap_or(false) {
            println!("[Browser Auth] Found confirm button, clicking...");
            automation.click_element(&tab, selector)?;
            std::thread::sleep(std::time::Duration::from_secs(3));
            break;
        }
    }

    println!("[Browser Auth] Checking for login page...");
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    let email_input_selectors = vec![
        "input[type='email']",
        "input[name='email']",
        "input[id*='email']",
        "input[placeholder*='example.com']",
        "input[placeholder*='username']",
        "input[placeholder*='email']",
        "input[placeholder*='Email']",
        "input[autocomplete='email']",
        "input[autocomplete='username']",
        "input[type='text']",
        "input:not([type='hidden']):not([type='submit']):not([type='button'])",
    ];
    
    let mut email_input_found = false;
    for selector in &email_input_selectors {
        println!("[Browser Auth] Trying email selector: {}", selector);
        if automation.wait_for_element(&tab, selector, 3).await.unwrap_or(false) {
            println!("[Browser Auth] Found email input at: {}", selector);
            automation.input_text(&tab, selector, email)?;
            email_input_found = true;
            std::thread::sleep(std::time::Duration::from_millis(1500));
            break;
        }
    }
    
    if !email_input_found {
        println!("[Browser Auth] Email input not found, checking if already logged in...");
        let allow_selectors = vec![
            "//button[contains(text(), 'Allow')]",
            "//button[contains(text(), 'Authorize')]",
            "//input[@type='submit' and contains(@value, 'Allow')]",
        ];
        
        for selector in &allow_selectors {
            if automation.wait_for_element(&tab, selector, 5).await.unwrap_or(false) {
                println!("[Browser Auth] Found allow button, clicking...");
                automation.click_element(&tab, selector)?;
                std::thread::sleep(std::time::Duration::from_secs(3));
                return Ok(());
            }
        }
    }
    
    let next_button_selectors = vec![
        "button[type='submit']",
        "input[type='submit']",
        "button.awsui-button-variant-primary",
        "button[data-testid='continue-button']",
        "button[data-testid='next-button']",
    ];
    
    println!("[Browser Auth] Looking for continue button...");
    for selector in &next_button_selectors {
        println!("[Browser Auth] Trying button selector: {}", selector);
        if automation.wait_for_element(&tab, selector, 3).await.unwrap_or(false) {
            println!("[Browser Auth] Clicking next/continue button: {}", selector);
            automation.click_element(&tab, selector)?;
            std::thread::sleep(std::time::Duration::from_secs(3));
            break;
        }
    }
    
    std::thread::sleep(std::time::Duration::from_secs(3));

    println!("[Browser Auth] Looking for password input...");
    let password_input_selectors = vec![
        "input[type='password']",
        "input[name='password']",
        "input[id*='password']",
        "input[autocomplete='current-password']",
    ];
    
    for selector in &password_input_selectors {
        println!("[Browser Auth] Trying password selector: {}", selector);
        if automation.wait_for_element(&tab, selector, 10).await.unwrap_or(false) {
            println!("[Browser Auth] Found password input, entering password...");
            automation.input_text(&tab, selector, kiro_password)?;
            std::thread::sleep(std::time::Duration::from_millis(1500));
            
            let signin_button_selectors = vec![
                "button[type='submit']",
                "input[type='submit']",
                "button.awsui-button-variant-primary",
            ];
            
            for btn_selector in &signin_button_selectors {
                if automation.wait_for_element(&tab, btn_selector, 3).await.unwrap_or(false) {
                    println!("[Browser Auth] Clicking sign in button: {}", btn_selector);
                    automation.click_element(&tab, btn_selector)?;
                    std::thread::sleep(std::time::Duration::from_secs(4));
                    break;
                }
            }
            
            std::thread::sleep(std::time::Duration::from_secs(4));
            break;
        }
    }

    println!("[Browser Auth] Checking for MFA/verification code...");
    let code_input_selectors = vec![
        "input[placeholder*='code']",
        "input[placeholder*='Code']",
        "input[placeholder*='验证码']",
        "input[placeholder*='verification']",
        "input[id*='code']",
        "input[name*='code']",
        "input[autocomplete='one-time-code']",
    ];
    
    for selector in &code_input_selectors {
        println!("[Browser Auth] Trying code selector: {}", selector);
        if automation.wait_for_element(&tab, selector, 5).await.unwrap_or(false) {
            println!("[Browser Auth] Found verification code input, fetching code from email...");
            
            let graph_client = GraphApiClient::new();
            match graph_client
                .wait_for_verification_code(email_client_id, email_refresh_token, email, 60)
                .await
            {
                Ok(verification_code) => {
                    println!("[Browser Auth] Got verification code, entering...");
                    automation.input_text(&tab, selector, &verification_code)?;
                    std::thread::sleep(std::time::Duration::from_millis(1500));
                    
                    std::thread::sleep(std::time::Duration::from_secs(3));
                }
                Err(e) => {
                    println!("[Browser Auth] Failed to get verification code: {}", e);
                }
            }
            break;
        }
    }

    println!("[Browser Auth] Looking for authorization button...");
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    std::thread::sleep(std::time::Duration::from_secs(3));

    println!("[Browser Auth] Waiting for authorization completion...");
    std::thread::sleep(std::time::Duration::from_secs(5));

    let _ = automation.clear_browser_data();

    Ok(())
}
