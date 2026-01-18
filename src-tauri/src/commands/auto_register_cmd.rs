use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResult {
    pub success: bool,
    pub email: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub error: Option<String>,
}

/// 自动注册 Kiro 账号
/// 
/// 此命令会：
/// 1. 打开浏览器访问 Kiro 注册页面
/// 2. 自动填写注册信息
/// 3. 完成注册后从浏览器提取 Token
/// 4. 返回账号信息
#[tauri::command]
pub async fn auto_register_kiro_account() -> Result<RegisterResult, String> {
    println!("[AutoRegister] 开始自动注册流程...");
    
    // 生成随机邮箱
    let random_email = generate_random_email();
    println!("[AutoRegister] 使用邮箱: {}", random_email);
    
    // 启动浏览器自动化注册
    match perform_browser_registration(&random_email).await {
        Ok(tokens) => {
            println!("[AutoRegister] 注册成功，已获取 Token");
            Ok(RegisterResult {
                success: true,
                email: Some(random_email),
                access_token: Some(tokens.access_token),
                refresh_token: Some(tokens.refresh_token),
                error: None,
            })
        }
        Err(e) => {
            eprintln!("[AutoRegister] 注册失败: {}", e);
            Ok(RegisterResult {
                success: false,
                email: None,
                access_token: None,
                refresh_token: None,
                error: Some(e),
            })
        }
    }
}

/// 导入已注册的账号到账号列表
#[tauri::command]
pub async fn import_registered_account(
    email: String,
    access_token: String,
    refresh_token: String,
) -> Result<(), String> {
    use crate::account::AccountStore;
    use crate::account::Account;
    
    println!("[AutoRegister] 导入账号: {}", email);
    
    let mut store = AccountStore::new();
    
    // 创建新账号
    let account = Account {
        id: uuid::Uuid::new_v4().to_string(),
        email: email.clone(),
        provider: "auto_register".to_string(),
        access_token: Some(access_token),
        refresh_token: Some(refresh_token),
        created_at: chrono::Utc::now().to_rfc3339(),
        last_used: None,
        tags: vec!["auto_registered".to_string()],
        notes: Some("通过自动注册功能创建".to_string()),
        ..Default::default()
    };
    
    store.add_account(account)?;
    store.save()?;
    
    println!("[AutoRegister] 账号导入成功");
    Ok(())
}

// ==================== 内部辅助函数 ====================

fn generate_random_email() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let random_str: String = (0..10)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect();
    format!("kiro_{}@temp-mail.io", random_str.to_lowercase())
}

#[derive(Debug)]
struct ExtractedTokens {
    access_token: String,
    refresh_token: String,
}

async fn perform_browser_registration(email: &str) -> Result<ExtractedTokens, String> {
    println!("[AutoRegister] 启动浏览器自动化...");
    
    // 使用 headless_chrome 进行浏览器自动化
    use headless_chrome::{Browser, LaunchOptions};
    use headless_chrome::protocol::cdp::Page;
    
    let browser = Browser::new(LaunchOptions {
        headless: false, // 显示浏览器窗口以便调试
        ..Default::default()
    }).map_err(|e| format!("启动浏览器失败: {}", e))?;
    
    let tab = browser.new_tab().map_err(|e| format!("创建标签页失败: {}", e))?;
    
    // 1. 访问 Kiro 注册页面
    println!("[AutoRegister] 访问注册页面...");
    tab.navigate_to("https://kiro.ai/signup")
        .map_err(|e| format!("导航失败: {}", e))?;
    
    tab.wait_for_element("input[type='email']")
        .map_err(|e| format!("等待页面加载失败: {}", e))?;
    
    sleep(Duration::from_secs(2)).await;
    
    // 2. 填写注册表单
    println!("[AutoRegister] 填写注册信息...");
    
    // 输入邮箱
    tab.wait_for_element("input[type='email']")
        .and_then(|elem| elem.click())
        .map_err(|e| format!("点击邮箱输入框失败: {}", e))?;
    
    tab.type_str(email)
        .map_err(|e| format!("输入邮箱失败: {}", e))?;
    
    sleep(Duration::from_millis(500)).await;
    
    // 点击注册按钮（使用 Google 登录）
    tab.wait_for_element("button[aria-label*='Google']")
        .and_then(|elem| elem.click())
        .map_err(|e| format!("点击 Google 登录失败: {}", e))?;
    
    println!("[AutoRegister] 等待 OAuth 流程完成...");
    sleep(Duration::from_secs(5)).await;
    
    // 3. 等待重定向回 Kiro 并提取 Token
    println!("[AutoRegister] 提取 Token...");
    
    // 从 localStorage 提取 Token
    let tokens = extract_tokens_from_browser(&tab)?;
    
    println!("[AutoRegister] Token 提取成功");
    
    Ok(tokens)
}

fn extract_tokens_from_browser(tab: &headless_chrome::Tab) -> Result<ExtractedTokens, String> {
    use headless_chrome::protocol::cdp::Runtime;
    
    // 执行 JavaScript 获取 localStorage 中的 Token
    let script = r#"
        (function() {
            const accessToken = localStorage.getItem('kiro_access_token') || 
                               localStorage.getItem('accessToken') ||
                               localStorage.getItem('auth_token');
            const refreshToken = localStorage.getItem('kiro_refresh_token') || 
                                localStorage.getItem('refreshToken') ||
                                localStorage.getItem('refresh_token');
            
            // 也尝试从 cookies 中获取
            const cookies = document.cookie.split(';').reduce((acc, cookie) => {
                const [key, value] = cookie.trim().split('=');
                acc[key] = value;
                return acc;
            }, {});
            
            return {
                accessToken: accessToken || cookies['WorkosCursorSessionToken'] || cookies['kiro_session'],
                refreshToken: refreshToken || cookies['refresh_token']
            };
        })();
    "#;
    
    let result = tab.evaluate(script, false)
        .map_err(|e| format!("执行 JavaScript 失败: {}", e))?;
    
    // 解析结果
    let value = result.value.ok_or("未获取到返回值")?;
    
    // 从 JSON 中提取 token
    let access_token = value.get("accessToken")
        .and_then(|v| v.as_str())
        .ok_or("未找到 access_token")?
        .to_string();
    
    let refresh_token = value.get("refreshToken")
        .and_then(|v| v.as_str())
        .unwrap_or(&access_token) // 如果没有 refresh_token，使用 access_token
        .to_string();
    
    if access_token.is_empty() {
        return Err("Token 为空".to_string());
    }
    
    Ok(ExtractedTokens {
        access_token,
        refresh_token,
    })
}
