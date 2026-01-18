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

            // 自动获取 Kiro 凭证
            println!("[Auto Register] Registration successful, now fetching Kiro credentials...");
            let credentials_result = perform_kiro_login(
                &account.email,
                &kiro_password,
                &account.client_id,
                &account.refresh_token,
                settings.browser_mode.clone(),
            ).await;

            match credentials_result {
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

                    println!("[Auto Register] Kiro credentials obtained successfully!");
                    Ok(format!("注册完成！密码: {}\n已自动获取 AWS Builder ID 凭证", kiro_password))
                }
                Err(e) => {
                    println!("[Auto Register] Failed to get Kiro credentials: {}", e);
                    Ok(format!("注册完成！密码: {}\n但获取凭证失败: {}", kiro_password, e))
                }
            }
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
pub async fn auto_register_start_batch_registration(
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
        // Format: email----password----client_id----refresh_token----kiro_password----status
        let kiro_pwd = account.kiro_password.as_deref().unwrap_or("");
        let status_str = match account.status {
            crate::auto_register::models::AccountStatus::NotRegistered => "not_registered",
            crate::auto_register::models::AccountStatus::InProgress => "in_progress",
            crate::auto_register::models::AccountStatus::Registered => "registered",
            crate::auto_register::models::AccountStatus::Error => "error",
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
pub async fn auto_register_fetch_latest_email(
    db: State<'_, DbState>,
    account_id: i64,
) -> Result<Vec<crate::auto_register::models::EmailMessage>, String> {
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
pub async fn auto_register_get_kiro_credentials(
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
pub async fn auto_register_batch_fetch_kiro_credentials(
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

/// 导入已注册账号到主账号列表
#[tauri::command]
pub async fn auto_register_import_to_main(
    db: State<'_, DbState>,
    app_state: State<'_, crate::state::AppState>,
) -> Result<String, String> {
    use crate::providers::IdcProvider;
    use crate::providers::RefreshMetadata;
    use crate::kiro::get_machine_id;
    use crate::codewhisperer_client::CodeWhispererClient;
    
    // 获取所有已注册且有凭证的账号
    let accounts = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let all_accounts = database::get_accounts_by_status(&conn, "registered").map_err(|e| e.to_string())?;
        all_accounts.into_iter()
            .filter(|a| a.kiro_refresh_token.is_some() && a.kiro_client_id.is_some() && a.kiro_client_secret.is_some())
            .collect::<Vec<_>>()
    };

    if accounts.is_empty() {
        return Ok("没有可导入的账号（需要先获取凭证）".to_string());
    }

    let mut success_count = 0;
    let mut error_count = 0;

    for account in accounts {
        let refresh_token = account.kiro_refresh_token.as_ref().unwrap();
        let client_id = account.kiro_client_id.as_ref().unwrap();
        let client_secret = account.kiro_client_secret.as_ref().unwrap();
        let region = "us-east-1";

        // 使用主项目的 IdcProvider 刷新 token
        let metadata = RefreshMetadata {
            client_id: Some(client_id.clone()),
            client_secret: Some(client_secret.clone()),
            region: Some(region.to_string()),
            ..Default::default()
        };

        let idc_provider = IdcProvider::new("BuilderId", region, None);
        
        match idc_provider.refresh_token(refresh_token, metadata).await {
            Ok(auth_result) => {
                // 获取 usage 数据
                let machine_id = get_machine_id();
                let cw_client = CodeWhispererClient::new(&machine_id);
                let usage_call = cw_client.get_usage_limits(&auth_result.access_token).await;
                let (usage, is_banned) = match &usage_call {
                    Ok(u) => (Some(u.clone()), false),
                    Err(e) if e.starts_with("BANNED:") => (None, true),
                    Err(_) => (None, false),
                };
                let usage_data = serde_json::to_value(&usage).unwrap_or(serde_json::Value::Null);
                
                let user_id = usage.as_ref()
                    .and_then(|u| u.user_info.as_ref())
                    .and_then(|u| u.user_id.clone());
                
                use sha2::{Digest, Sha256};
                let start_url = "https://view.awsapps.com/start";
                let mut hasher = Sha256::new();
                hasher.update(start_url.as_bytes());
                let client_id_hash = hex::encode(hasher.finalize());
                
                let expires_at = chrono::Local::now() + chrono::Duration::seconds(auth_result.expires_in);
                
                // 添加到主账号列表
                let mut store = app_state.store.lock().unwrap();
                
                // 检查是否已存在
                if let Some(existing) = store.accounts.iter_mut()
                    .find(|a| a.email == account.email && a.provider.as_deref() == Some("BuilderId")) {
                    // 更新现有账号
                    existing.access_token = Some(auth_result.access_token);
                    existing.refresh_token = Some(auth_result.refresh_token);
                    existing.user_id = user_id;
                    existing.expires_at = Some(expires_at.format("%Y/%m/%d %H:%M:%S").to_string());
                    existing.client_id = Some(client_id.clone());
                    existing.client_secret = Some(client_secret.clone());
                    existing.region = Some(region.to_string());
                    existing.client_id_hash = Some(client_id_hash);
                    existing.id_token = auth_result.id_token;
                    existing.sso_session_id = auth_result.sso_session_id;
                    existing.usage_data = Some(usage_data);
                    existing.status = if is_banned { "已封禁".to_string() } else { "正常".to_string() };
                } else {
                    // 添加新账号
                    let mut main_account = crate::account::Account::new(
                        account.email.clone(),
                        format!("Kiro BuilderId 账号 (自动注册)"),
                    );
                    main_account.access_token = Some(auth_result.access_token);
                    main_account.refresh_token = Some(auth_result.refresh_token);
                    main_account.provider = Some("BuilderId".to_string());
                    main_account.user_id = user_id;
                    main_account.expires_at = Some(expires_at.format("%Y/%m/%d %H:%M:%S").to_string());
                    main_account.client_id = Some(client_id.clone());
                    main_account.client_secret = Some(client_secret.clone());
                    main_account.region = Some(region.to_string());
                    main_account.client_id_hash = Some(client_id_hash);
                    main_account.id_token = auth_result.id_token;
                    main_account.sso_session_id = auth_result.sso_session_id;
                    main_account.usage_data = Some(usage_data);
                    main_account.status = if is_banned { "已封禁".to_string() } else { "正常".to_string() };
                    store.accounts.insert(0, main_account);
                }
                
                // 保存
                store.save_to_file();
                success_count += 1;
                println!("[Import] Successfully imported account: {}", account.email);
            }
            Err(e) => {
                println!("[Import] Failed to refresh token for {}: {}", account.email, e);
                error_count += 1;
            }
        }
    }

    Ok(format!(
        "导入完成！成功: {}, 失败: {}",
        success_count, error_count
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

