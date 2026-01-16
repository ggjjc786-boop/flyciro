use crate::database::{self, DbState};
use crate::models::*;
use crate::graph_api::GraphApiClient;
use crate::browser_automation::BrowserAutomation;
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
        // Format: email----password----client_id----refresh_token
        let line = format!(
            "{}----{}----{}----{}",
            account.email,
            account.email_password,
            account.client_id,
            account.refresh_token
        );
        lines.push(line);
    }

    Ok(lines.join("\n"))
}
