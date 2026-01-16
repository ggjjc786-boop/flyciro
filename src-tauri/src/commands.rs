use crate::database::{self, DbState};
use crate::models::*;
use crate::graph_api::GraphApiClient;
use crate::browser_automation::BrowserAutomation;
use crate::aws_sso_client::AWSSSOClient;
use tauri::State;
use anyhow::{Result, Context, anyhow};

#[tauri::command]
pub async fn get_accounts(
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
pub async fn add_account(
    db: State<'_, DbState>,
    account: NewAccount,
) -> Result<i64, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::insert_account(&conn, account).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_account(
    db: State<'_, DbState>,
    update: AccountUpdate,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::update_account(&conn, update).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_account(
    db: State<'_, DbState>,
    id: i64,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::delete_account(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_all_accounts(
    db: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::delete_all_accounts(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_accounts(
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

        // Validate email format
        if !email.contains('@') {
            error_count += 1;
            errors.push(ImportError {
                line_number: index + 1,
                content: line.to_string(),
                reason: "Invalid email address".to_string(),
            });
            continue;
        }

        // Validate that fields are not empty
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
pub async fn get_settings(
    db: State<'_, DbState>,
) -> Result<Settings, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::get_settings(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_settings(
    db: State<'_, DbState>,
    settings: Settings,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::update_settings(&conn, settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_registration(
    db: State<'_, DbState>,
    account_id: i64,
) -> Result<String, String> {
    // Get account details
    let account = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_account_by_id(&conn, account_id).map_err(|e| e.to_string())?
    };

    // Update status to in_progress
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

    // Get browser settings
    let settings = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_settings(&conn).map_err(|e| e.to_string())?
    };

    // Generate random name for registration
    let names = vec![
        "Zhang Wei", "Wang Fang", "Li Na", "Liu Yang", "Chen Jing",
        "Zhang Min", "Wang Lei", "Li Qiang", "Liu Min", "Chen Wei",
    ];
    let random_name = names[rand::random::<usize>() % names.len()];

    // Start registration process
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
            // Update account with success
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
            // Update account with error
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

    // Apply fingerprint protection
    automation.apply_fingerprint_protection(&tab)?;

    // Navigate to signin page
    tab.navigate_to("https://app.kiro.dev/signin")
        .context("Failed to navigate to signin page")?;
    tab.wait_until_navigated()?;

    std::thread::sleep(std::time::Duration::from_secs(3));

    // Click the third button (Google sign in button)
    let google_button_xpath = "/html/body/div[2]/div/div[1]/main/div/div/div/div/div/div/div/div[1]/button[3]";

    if automation.wait_for_element(&tab, google_button_xpath, 10).await? {
        automation.click_element(&tab, google_button_xpath)?;
        std::thread::sleep(std::time::Duration::from_secs(4));
    } else {
        return Err(anyhow!("Google sign-in button not found"));
    }

    // Wait for email input page
    let email_page_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div";

    if automation.wait_for_element(&tab, email_page_xpath, 15).await? {
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Input email
        let email_input_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div/div[2]/div/div[2]/div/div/div/div/div/input";
        automation.input_text(&tab, email_input_xpath, email)?;
        std::thread::sleep(std::time::Duration::from_millis(2000));

        // Click continue button
        let continue_button_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div/div[3]/button";
        automation.click_element(&tab, continue_button_xpath)?;
        std::thread::sleep(std::time::Duration::from_secs(4));
    } else {
        return Err(anyhow!("Email input page not found"));
    }

    // Wait for name input page
    let name_page_xpath = "/html/body/div[2]/div/div/div[2]/div/div/div/div[2]/div";

    if automation.wait_for_element(&tab, name_page_xpath, 15).await? {
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Input name
        let name_input_xpath = "/html/body/div[2]/div/div/div[1]/div/div/form/fieldset/div/div/div/div/div/div/div/div/div[3]/div/div[2]/div/div/div/div/div/input";
        automation.input_text(&tab, name_input_xpath, name)?;
        std::thread::sleep(std::time::Duration::from_millis(2000));

        // Click continue button
        let continue_button_xpath = "/html/body/div[2]/div/div/div[1]/div/div/form/fieldset/div/div/div/div/div/div/div/div/div[4]/button";
        automation.click_element(&tab, continue_button_xpath)?;
        std::thread::sleep(std::time::Duration::from_secs(4));
    } else {
        return Err(anyhow!("Name input page not found"));
    }

    // Wait for verification code page
    let verification_page_xpath = "/html/body/div[2]/div/div/div[1]/div/div/div[2]/form/fieldset/div/div/div/div/div/div";

    if automation.wait_for_element(&tab, verification_page_xpath, 15).await? {
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Fetch verification code using Graph API
        let graph_client = GraphApiClient::new();

        let verification_code = match graph_client
            .wait_for_verification_code(client_id, refresh_token, email, 60)
            .await
        {
            Ok(code) => code,
            Err(_) => {
                // If no code received after 60 seconds, click resend button
                let resend_button_xpath = "/html/body/div[2]/div/div/div[1]/div/div/div[2]/form/fieldset/div/div/div/div/div/div/div[3]/div/div[2]/div/div/div/div/div/div[1]/div/div[2]/button";
                automation.click_element(&tab, resend_button_xpath)?;
                std::thread::sleep(std::time::Duration::from_secs(5));

                // Wait again for verification code
                graph_client
                    .wait_for_verification_code(client_id, refresh_token, email, 60)
                    .await?
            }
        };

        // Input verification code
        let code_input_xpath = "/html/body/div[2]/div/div/div[1]/div/div/div[2]/form/fieldset/div/div/div/div/div/div/div[3]/div/div[2]/div/div/div/div/div/div[1]/div/div[1]/div/input";
        automation.input_text(&tab, code_input_xpath, &verification_code)?;
        std::thread::sleep(std::time::Duration::from_millis(2000));

        // Click continue button
        let continue_button_xpath = "/html/body/div[2]/div/div/div[1]/div/div/div[2]/form/fieldset/div/div/div/div/div/div/div[4]/button";
        automation.click_element(&tab, continue_button_xpath)?;
        std::thread::sleep(std::time::Duration::from_secs(4));
    } else {
        return Err(anyhow!("Verification code page not found"));
    }

    // Wait for password input page
    let password_page_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div[2]/div[3]/button";

    if automation.wait_for_element(&tab, password_page_xpath, 15).await? {
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Generate a secure random password
        let password = generate_secure_password();

        // Input password
        let password_input_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div[1]/div[3]/div/div[2]/div/div/div/div/div/span/span/div/input";
        automation.input_text(&tab, password_input_xpath, &password)?;
        std::thread::sleep(std::time::Duration::from_millis(2000));

        // Input confirm password
        let confirm_password_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div[1]/div[4]/div/div[2]/div/div/div/div/div/input";
        automation.input_text(&tab, confirm_password_xpath, &password)?;
        std::thread::sleep(std::time::Duration::from_millis(2000));

        // Click continue button
        let continue_button_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div[2]/div[3]/button";
        automation.click_element(&tab, continue_button_xpath)?;
        std::thread::sleep(std::time::Duration::from_secs(4));

        // Wait for success page
        let success_page_xpath = "/html/body/div[2]/div/div[1]/main/div/div[1]/div/div/div/div/div[2]";

        if automation.wait_for_element(&tab, success_page_xpath, 15).await? {
            // Registration successful
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
pub async fn start_batch_registration(
    db: State<'_, DbState>,
) -> Result<String, String> {
    // Get all accounts with status 'not_registered'
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

    // Get browser settings once
    let settings = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_settings(&conn).map_err(|e| e.to_string())?
    };

    // Process each account sequentially
    for account in accounts {
        // Update status to in_progress
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

        // Generate random name for registration
        let names = vec![
            "Zhang Wei", "Wang Fang", "Li Na", "Liu Yang", "Chen Jing",
            "Zhang Min", "Wang Lei", "Li Qiang", "Liu Min", "Chen Wei",
        ];
        let random_name = names[rand::random::<usize>() % names.len()];

        // Start registration process
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
                // Update account with success
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
                // Update account with error
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
pub async fn export_accounts(
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
        // Format: email----password----client_id----refresh_token----kiro_password----status
        let kiro_pwd = account.kiro_password.as_deref().unwrap_or("");
        let status_str = match account.status {
            crate::models::AccountStatus::NotRegistered => "not_registered",
            crate::models::AccountStatus::InProgress => "in_progress",
            crate::models::AccountStatus::Registered => "registered",
            crate::models::AccountStatus::Error => "error",
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

/// 获取账号最新邮件
#[tauri::command]
pub async fn fetch_latest_email(
    db: State<'_, DbState>,
    account_id: i64,
) -> Result<Vec<crate::models::EmailMessage>, String> {
    // 获取账号信息
    let account = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_account_by_id(&conn, account_id).map_err(|e| e.to_string())?
    };
    
    // 使用 Graph API 获取最新邮件
    let graph_client = GraphApiClient::new();
    
    // 获取 access token
    let access_token = graph_client
        .get_access_token(&account.client_id, &account.refresh_token)
        .await
        .map_err(|e| format!("获取访问令牌失败: {}", e))?;
    
    // 获取最新的 10 封邮件
    let emails = graph_client
        .fetch_recent_emails(&access_token, &account.email, 10)
        .await
        .map_err(|e| format!("获取邮件失败: {}", e))?;
    
    Ok(emails)
}

/// 获取 Kiro 凭证
#[tauri::command]
pub async fn get_kiro_credentials(
    db: State<'_, DbState>,
    account_id: i64,
) -> Result<String, String> {
    // 获取账号信息
    let account = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_account_by_id(&conn, account_id).map_err(|e| e.to_string())?
    };

    // 检查账号是否已注册
    if account.status != AccountStatus::Registered {
        return Err("账号尚未完成注册，请先完成注册".to_string());
    }

    // 获取浏览器设置
    let settings = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_settings(&conn).map_err(|e| e.to_string())?
    };

    // 执行获取凭证流程
    let result = perform_kiro_login(
        &account.email,
        account.kiro_password.as_deref().unwrap_or(""),
        &account.client_id,
        &account.refresh_token,
        settings.browser_mode,
    ).await;

    match result {
        Ok(credentials) => {
            // 更新账号凭证信息
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

/// 批量获取 Kiro 凭证
#[tauri::command]
pub async fn batch_fetch_kiro_credentials(
    db: State<'_, DbState>,
) -> Result<String, String> {
    // 获取所有已注册的账号
    let accounts = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_accounts_by_status(&conn, "registered").map_err(|e| e.to_string())?
    };

    // 过滤出没有 Kiro 凭证的账号
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

    // 获取浏览器设置
    let settings = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_settings(&conn).map_err(|e| e.to_string())?
    };

    // 依次处理每个账号
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

        // 每个账号之间稍作等待
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    Ok(format!(
        "批量获取凭证完成！总计: {}, 成功: {}, 失败: {}",
        total_count, success_count, error_count
    ))
}

/// 执行 Kiro 登录流程并获取凭证
async fn perform_kiro_login(
    email: &str,
    kiro_password: &str,
    email_client_id: &str,
    email_refresh_token: &str,
    browser_mode: BrowserMode,
) -> Result<KiroCredentials> {
    let start_url = "https://view.awsapps.com/start";
    let sso_client = AWSSSOClient::new("us-east-1");

    // Step 1: 注册设备客户端
    println!("[Kiro Login] Step 1: Registering device client...");
    let client_reg = sso_client.register_device_client(start_url).await
        .map_err(|e| anyhow!("注册设备客户端失败: {}", e))?;

    // Step 2: 发起设备授权
    println!("[Kiro Login] Step 2: Starting device authorization...");
    let device_auth = sso_client.start_device_authorization(
        &client_reg.client_id,
        &client_reg.client_secret,
        start_url,
    ).await
        .map_err(|e| anyhow!("发起设备授权失败: {}", e))?;

    // Step 3: 启动浏览器自动完成授权
    println!("[Kiro Login] Step 3: Launching browser for authorization...");
    let verification_url = device_auth.verification_uri_complete.as_ref()
        .unwrap_or(&device_auth.verification_uri);
    
    // 启动浏览器完成授权流程
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

    // Step 4: 轮询获取 Token
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

/// 在浏览器中完成授权流程
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

    // Apply fingerprint protection
    automation.apply_fingerprint_protection(&tab)?;

    // Navigate to verification URL
    println!("[Browser Auth] Navigating to: {}", verification_url);
    tab.navigate_to(verification_url)
        .context("Failed to navigate to verification URL")?;
    tab.wait_until_navigated()?;

    std::thread::sleep(std::time::Duration::from_secs(3));

    // Step 1: 等待并点击 "Confirm and continue" 按钮（设备授权确认页面）
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

    // Step 2: 检查是否在 AWS Builder ID 登录页面
    println!("[Browser Auth] Checking for login page...");
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    // 先打印页面信息帮助调试
    let page_info_script = r#"
        (function() {
            var inputs = document.querySelectorAll('input');
            var buttons = document.querySelectorAll('button');
            var info = {
                url: window.location.href,
                title: document.title,
                inputs: [],
                buttons: []
            };
            inputs.forEach(function(input) {
                info.inputs.push({
                    type: input.type,
                    name: input.name,
                    id: input.id,
                    placeholder: input.placeholder,
                    className: input.className
                });
            });
            buttons.forEach(function(btn) {
                info.buttons.push({
                    text: btn.textContent.trim(),
                    type: btn.type,
                    className: btn.className
                });
            });
            return JSON.stringify(info, null, 2);
        })()
    "#;
    
    match tab.evaluate(page_info_script, true) {
        Ok(result) => {
            if let Some(value) = result.value {
                println!("[Browser Auth] Page info: {}", value);
            }
        }
        Err(e) => {
            println!("[Browser Auth] Failed to get page info: {}", e);
        }
    }
    
    // AWS Builder ID 登录页面的邮箱输入框 - 优先使用 CSS 选择器
    let email_input_selectors = vec![
        // CSS selectors (优先)
        "input[type='email']",
        "input[name='email']",
        "input[id*='email']",
        "input[placeholder*='example.com']",
        "input[placeholder*='username']",
        "input[placeholder*='email']",
        "input[placeholder*='Email']",
        "input[autocomplete='email']",
        "input[autocomplete='username']",
        // Fallback: 页面上的第一个文本输入框
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
        // 可能已经登录，检查是否有授权按钮
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
    
    // Step 3: 点击下一步/继续按钮（邮箱输入后）
    let next_button_selectors = vec![
        // CSS selectors for "继续" / "Continue" / "Next" buttons
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
    
    // 如果上面的选择器都没找到，尝试用 JavaScript 查找包含特定文字的按钮
    let click_by_text_script = r#"
        (function() {
            var buttons = document.querySelectorAll('button, input[type="submit"]');
            var targetTexts = ['继续', 'Continue', 'Next', '下一步'];
            for (var i = 0; i < buttons.length; i++) {
                var btn = buttons[i];
                var text = btn.textContent || btn.value || '';
                for (var j = 0; j < targetTexts.length; j++) {
                    if (text.indexOf(targetTexts[j]) !== -1) {
                        btn.scrollIntoView({behavior: 'smooth', block: 'center'});
                        btn.click();
                        return 'Clicked: ' + text;
                    }
                }
            }
            return 'No button found';
        })()
    "#;
    
    match tab.evaluate(click_by_text_script, true) {
        Ok(result) => {
            if let Some(value) = result.value {
                println!("[Browser Auth] Click by text result: {}", value);
            }
        }
        Err(e) => {
            println!("[Browser Auth] Click by text failed: {}", e);
        }
    }
    
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Step 4: 输入密码
    println!("[Browser Auth] Looking for password input...");
    let password_input_selectors = vec![
        // CSS selectors
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
            
            // 点击登录按钮
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
            
            // 如果上面的选择器都没找到，尝试用 JavaScript 查找
            let click_signin_script = r#"
                (function() {
                    var buttons = document.querySelectorAll('button, input[type="submit"]');
                    var targetTexts = ['登录', '继续', 'Sign in', 'Login', 'Continue', 'Submit'];
                    for (var i = 0; i < buttons.length; i++) {
                        var btn = buttons[i];
                        var text = btn.textContent || btn.value || '';
                        for (var j = 0; j < targetTexts.length; j++) {
                            if (text.indexOf(targetTexts[j]) !== -1) {
                                btn.click();
                                return 'Clicked: ' + text;
                            }
                        }
                    }
                    return 'No button found';
                })()
            "#;
            
            match tab.evaluate(click_signin_script, true) {
                Ok(result) => {
                    if let Some(value) = result.value {
                        println!("[Browser Auth] Sign in click result: {}", value);
                    }
                }
                Err(_) => {}
            }
            
            std::thread::sleep(std::time::Duration::from_secs(4));
            break;
        }
    }

    // Step 5: 检查是否需要邮箱验证码 (MFA)
    println!("[Browser Auth] Checking for MFA/verification code...");
    let code_input_selectors = vec![
        // CSS selectors
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
            
            // 使用 Graph API 获取验证码
            let graph_client = GraphApiClient::new();
            match graph_client
                .wait_for_verification_code(email_client_id, email_refresh_token, email, 60)
                .await
            {
                Ok(verification_code) => {
                    println!("[Browser Auth] Got verification code, entering...");
                    automation.input_text(&tab, selector, &verification_code)?;
                    std::thread::sleep(std::time::Duration::from_millis(1500));
                    
                    // 点击验证/提交按钮
                    let verify_script = r#"
                        (function() {
                            var buttons = document.querySelectorAll('button, input[type="submit"]');
                            var targetTexts = ['验证', '继续', '提交', 'Verify', 'Submit', 'Continue'];
                            for (var i = 0; i < buttons.length; i++) {
                                var btn = buttons[i];
                                var text = btn.textContent || btn.value || '';
                                for (var j = 0; j < targetTexts.length; j++) {
                                    if (text.indexOf(targetTexts[j]) !== -1) {
                                        btn.click();
                                        return 'Clicked: ' + text;
                                    }
                                }
                            }
                            // If no text match, click any submit button
                            var submitBtn = document.querySelector('button[type="submit"], input[type="submit"]');
                            if (submitBtn) {
                                submitBtn.click();
                                return 'Clicked submit button';
                            }
                            return 'No button found';
                        })()
                    "#;
                    
                    match tab.evaluate(verify_script, true) {
                        Ok(result) => {
                            if let Some(value) = result.value {
                                println!("[Browser Auth] Verify click result: {}", value);
                            }
                        }
                        Err(_) => {}
                    }
                    
                    std::thread::sleep(std::time::Duration::from_secs(3));
                }
                Err(e) => {
                    println!("[Browser Auth] Failed to get verification code: {}", e);
                }
            }
            break;
        }
    }

    // Step 6: 检查并点击授权/允许按钮
    println!("[Browser Auth] Looking for authorization button...");
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    // 使用 JavaScript 查找授权按钮
    let allow_script = r#"
        (function() {
            var buttons = document.querySelectorAll('button, input[type="submit"]');
            var targetTexts = ['允许', '授权', 'Allow', 'Authorize', 'Grant', 'Confirm'];
            for (var i = 0; i < buttons.length; i++) {
                var btn = buttons[i];
                var text = btn.textContent || btn.value || '';
                for (var j = 0; j < targetTexts.length; j++) {
                    if (text.indexOf(targetTexts[j]) !== -1) {
                        btn.click();
                        return 'Clicked: ' + text;
                    }
                }
            }
            return 'No allow button found';
        })()
    "#;
    
    match tab.evaluate(allow_script, true) {
        Ok(result) => {
            if let Some(value) = result.value {
                println!("[Browser Auth] Allow button result: {}", value);
            }
        }
        Err(_) => {}
    }
    
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Step 7: 等待授权完成（成功页面或 URL 变化）
    println!("[Browser Auth] Waiting for authorization completion...");
    std::thread::sleep(std::time::Duration::from_secs(5));
    
    // 检查成功指示
    let check_success_script = r#"
        (function() {
            var body = document.body.textContent || '';
            var successTexts = ['success', 'Success', 'authorized', 'Authorized', 'complete', 'Complete', 'You can close', '成功', '授权完成'];
            for (var i = 0; i < successTexts.length; i++) {
                if (body.indexOf(successTexts[i]) !== -1) {
                    return 'Success: found "' + successTexts[i] + '"';
                }
            }
            return 'No success indicator found';
        })()
    "#;
    
    match tab.evaluate(check_success_script, true) {
        Ok(result) => {
            if let Some(value) = result.value {
                println!("[Browser Auth] Success check: {}", value);
            }
        }
        Err(_) => {}
    }

    // 清理浏览器数据
    let _ = automation.clear_browser_data();

    Ok(())
}



// ============ 卡密验证相关命令 ============

// RC4 解密函数
fn rc4_decrypt(key: &str, hex_data: &str) -> Result<String, String> {
    // 将十六进制字符串转换为字节
    let mut data = Vec::new();
    let mut i = 0;
    while i < hex_data.len() {
        if i + 2 <= hex_data.len() {
            let byte = u8::from_str_radix(&hex_data[i..i+2], 16)
                .map_err(|_| "无效的十六进制数据")?;
            data.push(byte);
            i += 2;
        } else {
            break;
        }
    }
    
    // RC4 解密
    let key_bytes = key.as_bytes();
    let mut s: Vec<u8> = (0..=255).collect();
    let mut j: usize = 0;
    
    // KSA (Key Scheduling Algorithm)
    for i in 0..256 {
        j = (j + s[i] as usize + key_bytes[i % key_bytes.len()] as usize) % 256;
        s.swap(i, j);
    }
    
    // PRGA (Pseudo-Random Generation Algorithm)
    let mut i: usize = 0;
    j = 0;
    let mut result = Vec::new();
    
    for byte in data {
        i = (i + 1) % 256;
        j = (j + s[i] as usize) % 256;
        s.swap(i, j);
        let t = (s[i] as usize + s[j] as usize) % 256;
        result.push(byte ^ s[t]);
    }
    
    String::from_utf8(result).map_err(|_| "解密后的数据不是有效的UTF-8".to_string())
}

#[derive(serde::Deserialize)]
pub struct KamiLoginRequest {
    pub kami: String,
    pub markcode: String,
}

#[derive(serde::Serialize)]
pub struct KamiLoginResponse {
    pub success: bool,
    pub message: String,
    pub vip_expire_time: Option<i64>,
}

const RC4_KEY: &str = "8HacPHMcsWK10002";

/// 卡密登录验证
#[tauri::command]
pub async fn kami_login(kami: String, markcode: String) -> Result<KamiLoginResponse, String> {
    let app_id = "10002";
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let url = format!(
        "https://zh.xphdfs.me/api.php?api=kmlogon&app={}&kami={}&markcode={}&t={}",
        app_id,
        urlencoding::encode(&kami),
        urlencoding::encode(&markcode),
        timestamp
    );
    
    println!("[卡密验证] 请求URL: {}", url);
    
    let client = reqwest::Client::new();
    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {}", e))?;
    
    // 获取原始响应文本
    let text = response.text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;
    
    println!("[卡密验证] 原始响应: {}", text);
    
    // 尝试直接解析 JSON，如果失败则尝试 RC4 解密
    let json_str = if text.starts_with('{') {
        text.clone()
    } else {
        // RC4 解密
        let decrypted = rc4_decrypt(RC4_KEY, &text)?;
        println!("[卡密验证] 解密后: {}", decrypted);
        decrypted
    };
    
    // 解析 JSON
    let json: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("解析JSON失败: {} - 内容: {}", e, json_str))?;
    
    println!("[卡密验证] JSON: {:?}", json);
    
    // code 是字符串类型，如 "200", "148" 等
    let code_str = json["code"].as_str().unwrap_or("");
    let code: i64 = code_str.parse().unwrap_or(0);
    
    println!("[卡密验证] 错误码字符串: {}, 解析后: {}", code_str, code);
    
    if code == 200 {
        // 成功时 msg 是对象 {"kami": "xxx", "vip": "timestamp"}
        let vip = json["msg"]["vip"].as_str()
            .and_then(|s| s.parse::<i64>().ok())
            .or_else(|| json["msg"]["vip"].as_i64());
        
        Ok(KamiLoginResponse {
            success: true,
            message: "验证成功".to_string(),
            vip_expire_time: vip,
        })
    } else {
        // 根据错误码返回对应的错误消息
        let msg = match code {
            101 => "应用不存在",
            102 => "应用已关闭",
            104 => "签名为空",
            105 => "数据过期",
            106 => "签名有误",
            148 => "卡密为空",
            149 => "卡密不存在",
            151 => "卡密禁用",
            169 => "IP不一致",
            171 => "接口维护中",
            172 => "接口未添加或不存在",
            _ => {
                // 尝试从 msg 字段获取错误信息
                if let Some(s) = json["msg"].as_str() {
                    return Ok(KamiLoginResponse {
                        success: false,
                        message: s.to_string(),
                        vip_expire_time: None,
                    });
                }
                "验证失败"
            }
        };
        Ok(KamiLoginResponse {
            success: false,
            message: format!("{} (错误码:{})", msg, code),
            vip_expire_time: None,
        })
    }
}

/// 卡密解绑
#[tauri::command]
pub async fn kami_unbind(markcode: String) -> Result<KamiLoginResponse, String> {
    let app_id = "10002";
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let url = format!(
        "https://zh.xphdfs.me/api.php?api=kmunmachine&app={}&markcode={}&t={}",
        app_id,
        urlencoding::encode(&markcode),
        timestamp
    );
    
    let client = reqwest::Client::new();
    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {}", e))?;
    
    let text = response.text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;
    
    // 尝试直接解析 JSON，如果失败则尝试 RC4 解密
    let json_str = if text.starts_with('{') {
        text.clone()
    } else {
        rc4_decrypt(RC4_KEY, &text)?
    };
    
    let json: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("解析JSON失败: {}", e))?;
    
    let code = json["code"].as_i64().unwrap_or(0);
    let msg = json["msg"].as_str().unwrap_or("操作完成").to_string();
    
    Ok(KamiLoginResponse {
        success: code == 200,
        message: msg,
        vip_expire_time: None,
    })
}

/// 获取公告
#[tauri::command]
pub async fn get_notice() -> Result<String, String> {
    let app_id = "10002";
    
    let url = format!(
        "https://zh.xphdfs.me/api.php?api=notice&app={}",
        app_id
    );
    
    let client = reqwest::Client::new();
    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {}", e))?;
    
    let text = response.text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;
    
    // 尝试直接解析 JSON，如果失败则尝试 RC4 解密
    let json_str = if text.starts_with('{') {
        text.clone()
    } else {
        rc4_decrypt(RC4_KEY, &text).unwrap_or_default()
    };
    
    let json: serde_json::Value = serde_json::from_str(&json_str)
        .unwrap_or(serde_json::Value::Null);
    
    let code = json["code"].as_i64().unwrap_or(0);
    
    if code == 200 {
        let notice = json["msg"]["app_gg"].as_str().unwrap_or("").to_string();
        Ok(notice)
    } else {
        Ok("".to_string())
    }
}
