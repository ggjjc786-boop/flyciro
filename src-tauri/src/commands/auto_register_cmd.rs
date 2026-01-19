use crate::auto_register::database::{self, DbState};
use crate::auto_register::models::*;
use crate::auto_register::graph_api::GraphApiClient;
use crate::auto_register::browser_automation::BrowserAutomation;
use crate::auto_register::aws_sso_client::AWSSSOClient;
use crate::providers::AuthProvider;
use crate::auth::User;
use tauri::{State, Emitter};
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
    app_handle: tauri::AppHandle,
    db: State<'_, DbState>,
    app_state: State<'_, crate::state::AppState>,
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
        settings.browser_mode.clone(),
    ).await;

    match result {
        Ok((kiro_password, automation, browser, tab)) => {
            // Update account with success
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
            } // conn 在这里被释放

            // 自动获取 Kiro 凭证
            println!("[Auto Register] Registration successful, now fetching Kiro credentials...");
            
            // 使用同一个浏览器会话完成 AWS SSO 授权
            let credentials_result = perform_kiro_login_with_browser(
                &account.email,
                &kiro_password,
                &account.client_id,
                &account.refresh_token,
                &automation,
                &browser,
                &tab,
            ).await;

            match credentials_result {
                Ok(credentials) => {
                    // 更新账号凭证信息
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
                    }

                    // 清理浏览器数据
                    let _ = automation.clear_browser_data();

                    println!("[Auto Register] Kiro credentials obtained successfully!");
                    println!("[Auto Register] Now importing to main account list...");
                    
                    // 导入到主账号列表
                    use crate::providers::IdcProvider;
                    use crate::providers::RefreshMetadata;
                    use crate::kiro::get_machine_id;
                    use crate::codewhisperer_client::CodeWhispererClient;

                    let region = "us-east-1";
                    let metadata = RefreshMetadata {
                        client_id: Some(credentials.client_id.clone()),
                        client_secret: Some(credentials.client_secret.clone()),
                        region: Some(region.to_string()),
                        ..Default::default()
                    };

                    let idc_provider = IdcProvider::new("BuilderId", region, None);
                    
                    match idc_provider.refresh_token(&credentials.refresh_token, metadata).await {
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
                            
                            // 先克隆需要多次使用的值
                            let access_token_clone = auth_result.access_token.clone();
                            let refresh_token_clone = auth_result.refresh_token.clone();
                            
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
                                existing.client_id = Some(credentials.client_id);
                                existing.client_secret = Some(credentials.client_secret);
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
                                main_account.client_id = Some(credentials.client_id);
                                main_account.client_secret = Some(credentials.client_secret);
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
                            
                            // 获取导入的账号 ID
                            let account_id_for_event = if let Some(acc) = store.accounts.iter().find(|a| a.email == account.email && a.provider.as_deref() == Some("BuilderId")) {
                                acc.id.clone()
                            } else {
                                String::new()
                            };
                            
                            drop(store); // 释放 store 锁
                            
                            // 更新 auth 状态，标记用户已登录
                            let user = User {
                                id: uuid::Uuid::new_v4().to_string(),
                                email: account.email.clone(),
                                name: account.email.split('@').next().unwrap_or("User").to_string(),
                                avatar: None,
                                provider: "BuilderId".to_string(),
                            };
                            *app_state.auth.user.lock().unwrap() = Some(user);
                            *app_state.auth.access_token.lock().unwrap() = Some(access_token_clone.clone());
                            *app_state.auth.refresh_token.lock().unwrap() = Some(refresh_token_clone.clone());
                            
                            // 发送登录成功事件，通知前端刷新
                            if !account_id_for_event.is_empty() {
                                let _ = app_handle.emit("login-success", account_id_for_event);
                                println!("[Auto Register] Emitted login-success event");
                            }
                            
                            println!("[Auto Register] Successfully imported to main account list!");
                            
                            Ok(format!("注册完成！密码: {}\n已自动获取 AWS Builder ID 凭证并导入到账号列表", kiro_password))
                        }
                        Err(e) => {
                            println!("[Auto Register] Failed to import to main account list: {}", e);
                            Ok(format!("注册完成！密码: {}\n已获取凭证但导入失败: {}", kiro_password, e))
                        }
                    }
                }
                Err(e) => {
                    // 清理浏览器数据
                    let _ = automation.clear_browser_data();
                    
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
) -> Result<(String, BrowserAutomation, headless_chrome::Browser, std::sync::Arc<headless_chrome::Tab>)> {
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
            // Registration successful - 不关闭浏览器，返回浏览器对象供后续使用
            println!("[Registration] Success! Keeping browser open for AWS SSO authorization...");
            return Ok((password, automation, browser, tab));
        } else {
            return Err(anyhow!("Success page not found - registration may have failed"));
        }
    } else {
        return Err(anyhow!("Password input page not found"));
    }
}

fn generate_secure_password() -> String {
    use rand::Rng;
    use rand::seq::SliceRandom;
    
    let mut rng = rand::thread_rng();
    
    // 定义字符集
    const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    const DIGITS: &[u8] = b"0123456789";
    const SPECIAL: &[u8] = b"!";
    const EXTRA_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    
    let mut password_chars = Vec::new();
    
    // 确保至少有一个大写字母
    let uppercase_idx = rng.gen_range(0..UPPERCASE.len());
    password_chars.push(UPPERCASE[uppercase_idx] as char);
    
    // 确保至少有一个小写字母
    let lowercase_idx = rng.gen_range(0..LOWERCASE.len());
    password_chars.push(LOWERCASE[lowercase_idx] as char);
    
    // 确保至少有一个数字
    let digit_idx = rng.gen_range(0..DIGITS.len());
    password_chars.push(DIGITS[digit_idx] as char);
    
    // 确保有一个感叹号
    password_chars.push('!');
    
    // 填充剩余字符（总长度16，已有4个必需字符，还需12个）
    for _ in 0..12 {
        let idx = rng.gen_range(0..EXTRA_CHARS.len());
        password_chars.push(EXTRA_CHARS[idx] as char);
    }
    
    // 随机打乱字符顺序
    password_chars.shuffle(&mut rng);
    
    password_chars.into_iter().collect()
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
            Ok((kiro_password, automation, _browser, _tab)) => {
                // 批量注册时不自动获取凭证，只保存密码
                // 清理浏览器数据
                let _ = automation.clear_browser_data();
                
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
        settings.browser_mode.clone(),
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

/// 获取单个账号的凭证并导入到主账号池
#[tauri::command]
pub async fn auto_register_get_credentials_and_import(
    app_handle: tauri::AppHandle,
    db: State<'_, DbState>,
    app_state: State<'_, crate::state::AppState>,
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
    println!("[Get Credentials] Starting to fetch credentials for account: {}", account.email);
    let result = perform_kiro_login(
        &account.email,
        account.kiro_password.as_deref().unwrap_or(""),
        &account.client_id,
        &account.refresh_token,
        settings.browser_mode.clone(),
    ).await;

    match result {
        Ok(credentials) => {
            // 更新自动注册数据库中的账号凭证信息
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
            }

            println!("[Get Credentials] Credentials obtained, now importing to main account pool...");

            // 导入到主账号池
            use crate::providers::IdcProvider;
            use crate::providers::RefreshMetadata;
            use crate::kiro::get_machine_id;
            use crate::codewhisperer_client::CodeWhispererClient;

            let region = "us-east-1";
            let metadata = RefreshMetadata {
                client_id: Some(credentials.client_id.clone()),
                client_secret: Some(credentials.client_secret.clone()),
                region: Some(region.to_string()),
                ..Default::default()
            };

            let idc_provider = IdcProvider::new("BuilderId", region, None);
            
            match idc_provider.refresh_token(&credentials.refresh_token, metadata).await {
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
                    
                    // 先克隆需要多次使用的值
                    let access_token_clone = auth_result.access_token.clone();
                    let refresh_token_clone = auth_result.refresh_token.clone();
                    
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
                        existing.client_id = Some(credentials.client_id);
                        existing.client_secret = Some(credentials.client_secret);
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
                        main_account.client_id = Some(credentials.client_id);
                        main_account.client_secret = Some(credentials.client_secret);
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
                    
                    // 获取导入的账号 ID
                    let account_id_for_event = if let Some(acc) = store.accounts.iter().find(|a| a.email == account.email && a.provider.as_deref() == Some("BuilderId")) {
                        acc.id.clone()
                    } else {
                        String::new()
                    };
                    
                    drop(store); // 释放 store 锁
                    
                    // 更新 auth 状态，标记用户已登录
                    let user = User {
                        id: uuid::Uuid::new_v4().to_string(),
                        email: account.email.clone(),
                        name: account.email.split('@').next().unwrap_or("User").to_string(),
                        avatar: None,
                        provider: "BuilderId".to_string(),
                    };
                    *app_state.auth.user.lock().unwrap() = Some(user);
                    *app_state.auth.access_token.lock().unwrap() = Some(access_token_clone.clone());
                    *app_state.auth.refresh_token.lock().unwrap() = Some(refresh_token_clone.clone());
                    
                    // 发送登录成功事件，通知前端刷新
                    if !account_id_for_event.is_empty() {
                        let _ = app_handle.emit("login-success", account_id_for_event);
                        println!("[Get Credentials] Emitted login-success event");
                    }
                    
                    println!("[Get Credentials] Successfully imported account: {}", account.email);
                    
                    Ok(format!("成功获取凭证并导入账号: {}", account.email))
                }
                Err(e) => {
                    println!("[Get Credentials] Failed to import: {}", e);
                    Err(format!("获取凭证成功但导入失败: {}", e))
                }
            }
        }
        Err(e) => {
            println!("[Get Credentials] Failed to get credentials: {}", e);
            Err(format!("获取凭证失败: {}", e))
        }
    }
}

/// 使用已登录的浏览器会话执行 Kiro 登录流程并获取凭证
async fn perform_kiro_login_with_browser(
    _email: &str,
    _kiro_password: &str,
    _email_client_id: &str,
    _email_refresh_token: &str,
    _automation: &BrowserAutomation,
    _browser: &headless_chrome::Browser,
    tab: &std::sync::Arc<headless_chrome::Tab>,
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

    // Step 3: 在已登录的浏览器中导航到授权页面
    println!("[Kiro Login] Step 3: Navigating to authorization page in logged-in browser...");
    let verification_url = device_auth.verification_uri_complete.as_ref()
        .unwrap_or(&device_auth.verification_uri);
    
    println!("[Kiro Login] Navigating to: {}", verification_url);
    tab.navigate_to(verification_url)
        .context("Failed to navigate to verification URL")?;
    tab.wait_until_navigated()?;
    
    std::thread::sleep(std::time::Duration::from_secs(3));
    
    // Step 4: 第一个页面 - 自动点击 "Confirm and continue" 按钮
    println!("[Kiro Login] Step 4: Auto-clicking 'Confirm and continue' button on first page...");
    
    // 使用 JavaScript 智能查找按钮（排除 Cookie 弹窗按钮）
    let find_button_script = r#"
        (function() {
            const buttons = document.querySelectorAll('button[type="submit"]');
            const targetTexts = ['Confirm and continue', '确认并继续'];
            const excludeTexts = ['Accept', 'Decline', 'Customize', 'Save preferences', 'Dismiss', 'Cancel', '取消'];
            
            for (let btn of buttons) {
                const text = btn.textContent.trim();
                if (targetTexts.some(target => text === target)) {
                    return true;
                }
            }
            return false;
        })()
    "#;
    
    let click_button_script = r#"
        (function() {
            const buttons = document.querySelectorAll('button[type="submit"]');
            const targetTexts = ['Confirm and continue', '确认并继续'];
            const excludeTexts = ['Accept', 'Decline', 'Customize', 'Save preferences', 'Dismiss', 'Cancel', '取消'];
            
            for (let btn of buttons) {
                const text = btn.textContent.trim();
                if (targetTexts.some(target => text === target)) {
                    btn.click();
                    return true;
                }
            }
            return false;
        })()
    "#;
    
    // 等待按钮出现
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(15);
    let mut button_found = false;
    
    while start.elapsed() < timeout {
        match tab.evaluate(find_button_script, true) {
            Ok(result) => {
                if let Some(value) = result.value {
                    if value.as_bool().unwrap_or(false) {
                        button_found = true;
                        break;
                    }
                }
            }
            Err(_) => {}
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    
    if !button_found {
        return Err(anyhow!("Could not find 'Confirm and continue' button"));
    }
    
    // 点击按钮
    println!("[Kiro Login] Found 'Confirm and continue' button, clicking...");
    tab.evaluate(click_button_script, true)
        .context("Failed to click 'Confirm and continue' button")?;
    println!("[Kiro Login] Successfully clicked 'Confirm and continue' button");
    std::thread::sleep(std::time::Duration::from_secs(4));
    
    // Step 5: 第二个页面 - 自动点击 "Allow access" 按钮
    println!("[Kiro Login] Step 5: Auto-clicking 'Allow access' button on second page...");
    
    // 使用 JavaScript 智能查找按钮
    let find_allow_script = r#"
        (function() {
            const buttons = document.querySelectorAll('button[type="submit"]');
            const targetTexts = ['Allow access', '允许访问'];
            const excludeTexts = ['Deny access', '拒绝访问', 'Cancel', '取消'];
            
            for (let btn of buttons) {
                const text = btn.textContent.trim();
                if (targetTexts.some(target => text === target)) {
                    return true;
                }
            }
            return false;
        })()
    "#;
    
    let click_allow_script = r#"
        (function() {
            const buttons = document.querySelectorAll('button[type="submit"]');
            const targetTexts = ['Allow access', '允许访问'];
            const excludeTexts = ['Deny access', '拒绝访问', 'Cancel', '取消'];
            
            for (let btn of buttons) {
                const text = btn.textContent.trim();
                if (targetTexts.some(target => text === target)) {
                    btn.click();
                    return true;
                }
            }
            return false;
        })()
    "#;
    
    // 等待按钮出现
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(15);
    let mut allow_button_found = false;
    
    while start.elapsed() < timeout {
        match tab.evaluate(find_allow_script, true) {
            Ok(result) => {
                if let Some(value) = result.value {
                    if value.as_bool().unwrap_or(false) {
                        allow_button_found = true;
                        break;
                    }
                }
            }
            Err(_) => {}
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    
    if !allow_button_found {
        return Err(anyhow!("Could not find 'Allow access' button"));
    }
    
    // 点击按钮
    println!("[Kiro Login] Found 'Allow access' button, clicking...");
    tab.evaluate(click_allow_script, true)
        .context("Failed to click 'Allow access' button")?;
    println!("[Kiro Login] Successfully clicked 'Allow access' button");
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    println!("[Kiro Login] Authorization buttons clicked successfully, waiting for token...");
    
    // Step 6: 轮询获取 Token
    println!("[Kiro Login] Step 6: Polling for token...");
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
                println!("[Kiro Login] ========================================");
                println!("[Kiro Login] 授权成功！Token obtained successfully!");
                println!("[Kiro Login] ========================================");
                return Ok(KiroCredentials {
                    client_id: client_reg.client_id,
                    client_secret: client_reg.client_secret,
                    refresh_token: token.refresh_token,
                    access_token: token.access_token,
                    id_token: token.id_token,
                });
            }
            Ok(DevicePollResult::Pending) => {
                println!("[Kiro Login] Waiting for authorization... (attempt {}/{}) - 等待用户授权中...", attempt + 1, max_attempts);
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

    Err(anyhow!("获取 Token 超时 - 用户未在规定时间内完成授权"))
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
    // 写入日志文件用于调试
    let log_file_path = std::env::temp_dir().join("kiro_browser_auth.log");
    let mut log_content = format!("=== Browser Authorization Log ===\n");
    log_content.push_str(&format!("Time: {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
    log_content.push_str(&format!("Verification URL: {}\n", verification_url));
    log_content.push_str(&format!("Email: {}\n\n", email));
    
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
    log_content.push_str(&format!("[Browser Auth] Navigating to: {}\n", verification_url));
    tab.navigate_to(verification_url)
        .context("Failed to navigate to verification URL")?;
    tab.wait_until_navigated()?;

    println!("[Browser Auth] Waiting for page to fully load...");
    log_content.push_str("[Browser Auth] Waiting for page to fully load...\n");
    std::thread::sleep(std::time::Duration::from_secs(5));
    
    // 打印当前 URL 用于调试
    let current_url_script = "window.location.href";
    match tab.evaluate(current_url_script, true) {
        Ok(result) => {
            if let Some(value) = result.value {
                println!("[Browser Auth] Current URL: {}", value);
                log_content.push_str(&format!("[Browser Auth] Current URL: {}\n", value));
            }
        }
        Err(e) => {
            println!("[Browser Auth] Failed to get current URL: {}", e);
            log_content.push_str(&format!("[Browser Auth] Failed to get current URL: {}\n", e));
        }
    }
    
    // 保存日志到文件
    let _ = std::fs::write(&log_file_path, &log_content);
    println!("[Browser Auth] Log file: {:?}", log_file_path);

    // Step 1: 等待并点击 "Confirm and continue" 按钮（设备授权确认页面）
    println!("[Browser Auth] Looking for confirm and continue button...");
    log_content.push_str("[Browser Auth] Looking for confirm and continue button...\n");
    let confirm_button_xpath = "//*[@id='cli_verification_btn']";
    
    if automation.wait_for_element(&tab, confirm_button_xpath, 5).await.unwrap_or(false) {
        println!("[Browser Auth] Found confirm button, clicking...");
        log_content.push_str("[Browser Auth] Found confirm button, clicking...\n");
        automation.click_element(&tab, confirm_button_xpath)?;
        std::thread::sleep(std::time::Duration::from_secs(3));
    } else {
        println!("[Browser Auth] Confirm button not found, might be on login page already");
        log_content.push_str("[Browser Auth] Confirm button not found, might be on login page already\n");
    }

    // Step 1.5: 检查是否在 AWS 通用登录页面，需要点击 "AWS Builder ID" 链接
    println!("[Browser Auth] Checking if we need to click AWS Builder ID link...");
    log_content.push_str("[Browser Auth] Checking if we need to click AWS Builder ID link...\n");
    
    // 尝试查找 AWS Builder ID 链接
    let builder_id_link_script = r#"
        (function() {
            var links = document.querySelectorAll('a');
            for (var i = 0; i < links.length; i++) {
                var text = links[i].textContent || links[i].innerText || '';
                if (text.indexOf('AWS Builder ID') !== -1 || text.indexOf('Builder ID') !== -1) {
                    links[i].click();
                    return 'Clicked AWS Builder ID link';
                }
            }
            return 'AWS Builder ID link not found';
        })()
    "#;
    
    match tab.evaluate(builder_id_link_script, true) {
        Ok(result) => {
            if let Some(value) = result.value {
                println!("[Browser Auth] AWS Builder ID link result: {}", value);
                log_content.push_str(&format!("[Browser Auth] AWS Builder ID link result: {}\n", value));
                if value.as_str().unwrap_or("").contains("Clicked") {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                }
            }
        }
        Err(e) => {
            println!("[Browser Auth] Error checking for AWS Builder ID link: {}", e);
            log_content.push_str(&format!("[Browser Auth] Error checking for AWS Builder ID link: {}\n", e));
        }
    }
    
    // 保存中间日志
    let _ = std::fs::write(&log_file_path, &log_content);

    // Step 2: 在 AWS Builder ID 登录页面输入邮箱
    println!("[Browser Auth] Waiting for email input on AWS Builder ID login page...");
    log_content.push_str("[Browser Auth] Waiting for email input on AWS Builder ID login page...\n");
    log_content.push_str(&format!("[Browser Auth] Email to input: {}\n", email));
    
    // 先等待一下确保页面元素加载完成
    std::thread::sleep(std::time::Duration::from_secs(3));
    
    // 先找到输入框的 ID
    let find_input_id_script = r#"
        (function() {
            var emailInput = document.querySelector('input[placeholder*="example.com"]') ||
                           document.querySelector('input[placeholder*="username"]');
            if (emailInput && emailInput.id) {
                return emailInput.id;
            }
            return '';
        })()
    "#;
    
    let input_id = match tab.evaluate(find_input_id_script, true) {
        Ok(result) => {
            if let Some(value) = result.value {
                let id = value.as_str().unwrap_or("");
                println!("[Browser Auth] Found input ID: {}", id);
                log_content.push_str(&format!("[Browser Auth] Found input ID: {}\n", id));
                id.to_string()
            } else {
                String::new()
            }
        }
        Err(e) => {
            println!("[Browser Auth] Failed to find input ID: {}", e);
            log_content.push_str(&format!("[Browser Auth] Failed to find input ID: {}\n", e));
            String::new()
        }
    };
    
    if !input_id.is_empty() {
        // 使用 XPath 通过 ID 定位并输入
        let email_xpath = format!("//*[@id='{}']", input_id);
        println!("[Browser Auth] Using XPath: {}", email_xpath);
        log_content.push_str(&format!("[Browser Auth] Using XPath: {}\n", email_xpath));
        
        match automation.input_text(&tab, &email_xpath, email) {
            Ok(_) => {
                println!("[Browser Auth] Email input successful via XPath");
                log_content.push_str("[Browser Auth] Email input successful via XPath\n");
            }
            Err(e) => {
                println!("[Browser Auth] Email input failed via XPath: {}", e);
                log_content.push_str(&format!("[Browser Auth] Email input failed via XPath: {}\n", e));
            }
        }
    } else {
        println!("[Browser Auth] WARNING: Could not find input ID");
        log_content.push_str("[Browser Auth] WARNING: Could not find input ID\n");
    }
    
    // 保存中间日志
    let _ = std::fs::write(&log_file_path, &log_content);
    
    std::thread::sleep(std::time::Duration::from_millis(2000));
    
    // 点击继续按钮
    println!("[Browser Auth] Clicking continue/next button...");
    log_content.push_str("[Browser Auth] Clicking continue/next button...\n");
    let click_continue_script = r#"
        (function() {
            var buttons = document.querySelectorAll('button');
            for (var i = 0; i < buttons.length; i++) {
                var text = buttons[i].textContent || buttons[i].innerText || '';
                if (text.indexOf('继续') !== -1 || text.indexOf('Continue') !== -1 || text.indexOf('Next') !== -1) {
                    buttons[i].click();
                    return 'clicked';
                }
            }
            // 如果没找到文本匹配的，尝试找 submit 按钮
            var submitBtn = document.querySelector('button[type="submit"]');
            if (submitBtn) {
                submitBtn.click();
                return 'clicked_submit';
            }
            return 'not_found';
        })()
    "#;
    
    match tab.evaluate(click_continue_script, true) {
        Ok(result) => {
            if let Some(value) = result.value {
                println!("[Browser Auth] Continue button click result: {}", value);
                log_content.push_str(&format!("[Browser Auth] Continue button click result: {}\n", value));
            }
        }
        Err(e) => {
            println!("[Browser Auth] Failed to click continue: {}", e);
            log_content.push_str(&format!("[Browser Auth] Failed to click continue: {}\n", e));
        }
    }
    
    // 保存中间日志
    let _ = std::fs::write(&log_file_path, &log_content);
    
    std::thread::sleep(std::time::Duration::from_secs(4));

    // Step 3: 等待密码输入页面并输入密码
    println!("[Browser Auth] Waiting for password input...");
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    // 转义密码以防止 JavaScript 语法错误
    let escaped_password = kiro_password.replace("\\", "\\\\").replace("\"", "\\\"").replace("'", "\\'");
    
    let input_password_script = format!(
        r#"
        (function() {{
            var passwordInput = document.querySelector('input[type="password"]') || 
                              document.querySelector('input[name="password"]');
            if (passwordInput) {{
                passwordInput.focus();
                
                // 使用 React 的方式触发事件
                var nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set;
                nativeInputValueSetter.call(passwordInput, "{}");
                
                passwordInput.dispatchEvent(new Event('input', {{ bubbles: true }}));
                passwordInput.dispatchEvent(new Event('change', {{ bubbles: true }}));
                passwordInput.dispatchEvent(new Event('blur', {{ bubbles: true }}));
                return 'success';
            }}
            return 'failed';
        }})()
        "#,
        escaped_password
    );
    
    match tab.evaluate(&input_password_script, true) {
        Ok(result) => {
            if let Some(value) = result.value {
                println!("[Browser Auth] Password input result: {}", value);
                if value.as_str().unwrap_or("") != "success" {
                    return Err(anyhow!("Failed to input password"));
                }
            }
        }
        Err(e) => {
            println!("[Browser Auth] Failed to input password: {}", e);
            return Err(anyhow!("Failed to input password: {}", e));
        }
    }
    
    std::thread::sleep(std::time::Duration::from_millis(2000));
    
    // 点击登录按钮
    println!("[Browser Auth] Clicking sign in button...");
    let click_signin_script = r#"
        (function() {
            var buttons = document.querySelectorAll('button');
            for (var i = 0; i < buttons.length; i++) {
                var text = buttons[i].textContent || buttons[i].innerText || '';
                if (text.indexOf('登录') !== -1 || text.indexOf('Sign in') !== -1 || text.indexOf('Sign In') !== -1 || text.indexOf('继续') !== -1 || text.indexOf('Continue') !== -1) {
                    buttons[i].click();
                    return 'clicked';
                }
            }
            // 如果没找到文本匹配的，尝试找 submit 按钮
            var submitBtn = document.querySelector('button[type="submit"]');
            if (submitBtn) {
                submitBtn.click();
                return 'clicked_submit';
            }
            return 'not_found';
        })()
    "#;
    
    match tab.evaluate(click_signin_script, true) {
        Ok(result) => {
            if let Some(value) = result.value {
                println!("[Browser Auth] Sign in button click result: {}", value);
            }
        }
        Err(e) => {
            println!("[Browser Auth] Failed to click sign in: {}", e);
        }
    }
    
    std::thread::sleep(std::time::Duration::from_secs(4));

    // Step 4: 检查是否需要验证码
    println!("[Browser Auth] Checking for verification code requirement...");
    let code_input_xpath = "/html/body/div/div/main/div/div/form/div[1]/div/awsui-input/div/div[1]/div[1]/div/input";
    
    if automation.wait_for_element(&tab, code_input_xpath, 8).await.unwrap_or(false) {
        println!("[Browser Auth] Verification code required, fetching from email...");
        
        let graph_client = GraphApiClient::new();
        match graph_client
            .wait_for_verification_code(email_client_id, email_refresh_token, email, 60)
            .await
        {
            Ok(verification_code) => {
                println!("[Browser Auth] Got verification code: {}, entering...", verification_code);
                automation.input_text(&tab, code_input_xpath, &verification_code)?;
                std::thread::sleep(std::time::Duration::from_millis(2000));

                // 点击验证按钮
                let verify_button_xpath = "/html/body/div/div/main/div/div/form/div[2]/div/div/awsui-button/button";
                println!("[Browser Auth] Clicking verify button...");
                automation.click_element(&tab, verify_button_xpath)?;
                std::thread::sleep(std::time::Duration::from_secs(4));
            }
            Err(e) => {
                println!("[Browser Auth] Failed to get verification code: {}", e);
                return Err(anyhow!("Failed to get verification code: {}", e));
            }
        }
    } else {
        println!("[Browser Auth] No verification code required");
    }

    // Step 5: 等待并点击允许/授权按钮
    println!("[Browser Auth] Looking for allow/authorize button...");
    let allow_button_xpath = "/html/body/div/div/main/div/div/form/div[2]/span/span/awsui-button/button";
    
    if automation.wait_for_element(&tab, allow_button_xpath, 15).await.unwrap_or(false) {
        println!("[Browser Auth] Found allow button, clicking...");
        automation.click_element(&tab, allow_button_xpath)?;
        std::thread::sleep(std::time::Duration::from_secs(3));
    } else {
        println!("[Browser Auth] Allow button not found, authorization might have completed automatically");
    }

    // 等待授权完成
    println!("[Browser Auth] Waiting for authorization to complete...");
    log_content.push_str("[Browser Auth] Waiting for authorization to complete...\n");
    std::thread::sleep(std::time::Duration::from_secs(5));
    
    // 清理浏览器数据
    let _ = automation.clear_browser_data();

    log_content.push_str("[Browser Auth] Browser authorization completed successfully\n");
    log_content.push_str(&format!("\n=== End of Log ===\n"));
    
    // 保存最终日志
    if let Err(e) = std::fs::write(&log_file_path, &log_content) {
        println!("[Browser Auth] Failed to write log file: {}", e);
    } else {
        println!("[Browser Auth] Log saved to: {:?}", log_file_path);
    }

    println!("[Browser Auth] Browser authorization completed successfully");
    Ok(())
}

