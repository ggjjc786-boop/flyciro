// 注册账号命令模块 - 使用 CDP (Chrome DevTools Protocol) 自动化注册

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};

// 全局状态
static REGISTRATION_RUNNING: AtomicBool = AtomicBool::new(false);
static VERIFICATION_CODE: Mutex<Option<String>> = Mutex::new(None);
static BROWSER_PROCESS: Mutex<Option<Child>> = Mutex::new(None);
static CDP_MESSAGE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationProgress {
    pub step: String,
    pub message: String,
    #[serde(rename = "type")]
    pub msg_type: String,
}

#[derive(Debug, Deserialize)]
struct CdpTarget {
    #[serde(rename = "webSocketDebuggerUrl")]
    web_socket_debugger_url: Option<String>,
    #[serde(rename = "type")]
    target_type: String,
}

fn emit_progress(app: &AppHandle, step: &str, message: &str, msg_type: &str) {
    let _ = app.emit("register-progress", RegistrationProgress {
        step: step.to_string(),
        message: message.to_string(),
        msg_type: msg_type.to_string(),
    });
}

fn should_stop() -> bool {
    !REGISTRATION_RUNNING.load(Ordering::SeqCst)
}

fn next_id() -> u64 {
    CDP_MESSAGE_ID.fetch_add(1, Ordering::SeqCst)
}

/// 启动浏览器
fn start_browser(browser_path: &str, incognito_arg: &str, debug_port: u16) -> Result<Child, String> {
    let mut args = vec![
        format!("--remote-debugging-port={}", debug_port),
        "--no-first-run".to_string(),
        "--no-default-browser-check".to_string(),
        "--disable-sync".to_string(),
        "--disable-translate".to_string(),
        "--disable-background-networking".to_string(),
        "--disable-component-update".to_string(),
        "--disable-features=TranslateUI".to_string(),
        "--user-data-dir=".to_string() + &std::env::temp_dir().join("kiro_register_profile").to_string_lossy(),
    ];
    
    if !incognito_arg.is_empty() {
        args.push(incognito_arg.to_string());
    }
    
    args.push("about:blank".to_string());
    
    println!("[Register] Starting browser: {} {:?}", browser_path, args);
    
    Command::new(browser_path)
        .args(&args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("启动浏览器失败: {}", e))
}

/// 获取 CDP WebSocket URL
async fn get_cdp_ws_url(debug_port: u16) -> Result<String, String> {
    let url = format!("http://127.0.0.1:{}/json", debug_port);
    
    for attempt in 1..=15 {
        match reqwest::get(&url).await {
            Ok(resp) => {
                if let Ok(targets) = resp.json::<Vec<CdpTarget>>().await {
                    for target in targets {
                        if target.target_type == "page" {
                            if let Some(ws_url) = target.web_socket_debugger_url {
                                return Ok(ws_url);
                            }
                        }
                    }
                }
            }
            Err(_) => {}
        }
        
        if attempt < 15 {
            sleep(Duration::from_secs(1)).await;
        }
    }
    
    Err("无法连接到浏览器调试端口".to_string())
}

#[tauri::command]
pub async fn start_registration(
    app: AppHandle,
    email: String,
    password: String,
    name: String,
    browser_path: String,
    incognito_arg: String,
) -> Result<(), String> {
    if REGISTRATION_RUNNING.load(Ordering::SeqCst) {
        return Err("注册流程已在运行中".to_string());
    }
    
    REGISTRATION_RUNNING.store(true, Ordering::SeqCst);
    
    {
        let mut code = VERIFICATION_CODE.lock().unwrap();
        *code = None;
    }

    let app_clone = app.clone();
    tokio::spawn(async move {
        if let Err(e) = run_registration(app_clone.clone(), email, password, name, browser_path, incognito_arg).await {
            emit_progress(&app_clone, "error", &format!("注册失败: {}", e), "error");
        }
        REGISTRATION_RUNNING.store(false, Ordering::SeqCst);
    });
    
    Ok(())
}

async fn run_registration(
    app: AppHandle,
    email: String,
    password: String,
    name: String,
    browser_path: String,
    incognito_arg: String,
) -> Result<(), String> {
    let debug_port = 9333u16; // 使用不同的端口避免冲突
    
    emit_progress(&app, "starting", "正在启动浏览器...", "info");
    
    // 启动浏览器
    let child = start_browser(&browser_path, &incognito_arg, debug_port)?;
    {
        let mut process = BROWSER_PROCESS.lock().unwrap();
        *process = Some(child);
    }
    
    emit_progress(&app, "browser_started", "✅ 浏览器已启动", "success");
    
    // 等待浏览器启动
    sleep(Duration::from_secs(2)).await;
    
    // 获取 WebSocket URL
    emit_progress(&app, "connecting", "正在连接浏览器...", "info");
    let ws_url = get_cdp_ws_url(debug_port).await?;
    
    emit_progress(&app, "connected", "✅ 已连接到浏览器", "success");
    
    // 连接 WebSocket
    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .map_err(|e| format!("WebSocket 连接失败: {}", e))?;
    
    let (mut write, mut read) = ws_stream.split();
    
    // 启用页面事件
    let enable_msg = json!({
        "id": next_id(),
        "method": "Page.enable"
    });
    write.send(Message::Text(enable_msg.to_string())).await.map_err(|e| e.to_string())?;
    let _ = read.next().await;
    
    // 导航到 Kiro
    emit_progress(&app, "navigating", "正在打开 Kiro 官网...", "info");
    
    let navigate_msg = json!({
        "id": next_id(),
        "method": "Page.navigate",
        "params": { "url": "https://kiro.dev" }
    });
    write.send(Message::Text(navigate_msg.to_string())).await.map_err(|e| e.to_string())?;
    let _ = read.next().await;
    
    sleep(Duration::from_secs(4)).await;
    if should_stop() { return Ok(()); }
    
    emit_progress(&app, "page_loaded", "✅ 页面已加载", "success");
    
    // 点击 Sign In
    emit_progress(&app, "clicking_signin", "正在点击 Sign In...", "info");
    
    let click_signin_js = r#"
        (function() {
            const links = document.querySelectorAll('a, button');
            for (const link of links) {
                const text = link.textContent.toLowerCase();
                if (text.includes('sign in') || text.includes('signin') || text.includes('login')) {
                    link.click();
                    return 'clicked';
                }
            }
            return 'not found';
        })()
    "#;
    
    let eval_msg = json!({
        "id": next_id(),
        "method": "Runtime.evaluate",
        "params": { "expression": click_signin_js }
    });
    write.send(Message::Text(eval_msg.to_string())).await.map_err(|e| e.to_string())?;
    let _ = read.next().await;
    
    sleep(Duration::from_secs(3)).await;
    if should_stop() { return Ok(()); }
    
    emit_progress(&app, "signin_clicked", "✅ 已点击登录", "success");
    
    // 点击 Builder ID
    emit_progress(&app, "clicking_builderid", "正在选择 Builder ID...", "info");
    
    let click_builderid_js = r#"
        (function() {
            const elements = document.querySelectorAll('a, button, div, span');
            for (const el of elements) {
                const text = el.textContent.toLowerCase();
                if (text.includes('builder id') || text.includes('builderid')) {
                    el.click();
                    return 'clicked builder id';
                }
            }
            // 尝试点击 Create one
            for (const el of elements) {
                const text = el.textContent.toLowerCase();
                if (text.includes('create') && (text.includes('one') || text.includes('account'))) {
                    el.click();
                    return 'clicked create';
                }
            }
            return 'not found';
        })()
    "#;
    
    let eval_msg = json!({
        "id": next_id(),
        "method": "Runtime.evaluate",
        "params": { "expression": click_builderid_js }
    });
    write.send(Message::Text(eval_msg.to_string())).await.map_err(|e| e.to_string())?;
    let _ = read.next().await;
    
    sleep(Duration::from_secs(3)).await;
    if should_stop() { return Ok(()); }
    
    // 输入邮箱
    emit_progress(&app, "entering_email", &format!("正在输入邮箱: {}", email), "info");
    
    let enter_email_js = format!(r#"
        (function() {{
            const inputs = document.querySelectorAll('input[type="email"], input[name="email"], input[id*="email"], input[placeholder*="email"], input[type="text"]');
            for (const input of inputs) {{
                if (input.offsetParent !== null) {{
                    input.focus();
                    input.value = '{}';
                    input.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    input.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    return 'entered';
                }}
            }}
            return 'not found';
        }})()
    "#, email);
    
    let eval_msg = json!({
        "id": next_id(),
        "method": "Runtime.evaluate",
        "params": { "expression": enter_email_js }
    });
    write.send(Message::Text(eval_msg.to_string())).await.map_err(|e| e.to_string())?;
    let _ = read.next().await;
    
    sleep(Duration::from_secs(1)).await;
    
    emit_progress(&app, "email_entered", "✅ 邮箱已输入", "success");
    
    // 点击继续
    emit_progress(&app, "clicking_next", "正在点击继续...", "info");
    
    let click_next_js = r#"
        (function() {
            const buttons = document.querySelectorAll('button, input[type="submit"]');
            for (const btn of buttons) {
                const text = btn.textContent.toLowerCase();
                if (text.includes('next') || text.includes('continue') || text.includes('继续')) {
                    btn.click();
                    return 'clicked';
                }
            }
            return 'not found';
        })()
    "#;
    
    let eval_msg = json!({
        "id": next_id(),
        "method": "Runtime.evaluate",
        "params": { "expression": click_next_js }
    });
    write.send(Message::Text(eval_msg.to_string())).await.map_err(|e| e.to_string())?;
    let _ = read.next().await;
    
    sleep(Duration::from_secs(3)).await;
    if should_stop() { return Ok(()); }
    
    // 输入姓名
    emit_progress(&app, "entering_name", &format!("正在输入姓名: {}", name), "info");
    
    let enter_name_js = format!(r#"
        (function() {{
            const inputs = document.querySelectorAll('input[name="name"], input[id*="name"], input[placeholder*="name"], input[type="text"]');
            for (const input of inputs) {{
                if (input.offsetParent !== null && !input.value) {{
                    input.focus();
                    input.value = '{}';
                    input.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    input.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    return 'entered';
                }}
            }}
            return 'not found';
        }})()
    "#, name);
    
    let eval_msg = json!({
        "id": next_id(),
        "method": "Runtime.evaluate",
        "params": { "expression": enter_name_js }
    });
    write.send(Message::Text(eval_msg.to_string())).await.map_err(|e| e.to_string())?;
    let _ = read.next().await;
    
    sleep(Duration::from_secs(1)).await;
    
    emit_progress(&app, "name_entered", "✅ 姓名已输入", "success");
    
    // 点击继续
    let eval_msg = json!({
        "id": next_id(),
        "method": "Runtime.evaluate",
        "params": { "expression": click_next_js }
    });
    write.send(Message::Text(eval_msg.to_string())).await.map_err(|e| e.to_string())?;
    let _ = read.next().await;
    
    sleep(Duration::from_secs(3)).await;
    if should_stop() { return Ok(()); }
    
    // 等待验证码
    emit_progress(&app, "waiting_for_code", "⏳ 请检查邮箱获取验证码，在左侧输入框中填入", "warning");
    
    let mut verification_code = String::new();
    for _ in 0..600 {
        if should_stop() { return Ok(()); }
        
        {
            let code = VERIFICATION_CODE.lock().unwrap();
            if let Some(c) = code.as_ref() {
                verification_code = c.clone();
                break;
            }
        }
        
        sleep(Duration::from_secs(1)).await;
    }
    
    if verification_code.is_empty() {
        emit_progress(&app, "timeout", "⏰ 等待验证码超时", "error");
        return Err("等待验证码超时".to_string());
    }
    
    emit_progress(&app, "code_received", &format!("✅ 收到验证码: {}", verification_code), "success");
    
    // 输入验证码
    emit_progress(&app, "entering_code", "正在输入验证码...", "info");
    
    let enter_code_js = format!(r#"
        (function() {{
            const inputs = document.querySelectorAll('input[name="code"], input[name="confirmationCode"], input[id*="code"], input[type="text"], input[type="number"]');
            for (const input of inputs) {{
                if (input.offsetParent !== null && !input.value) {{
                    input.focus();
                    input.value = '{}';
                    input.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    input.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    return 'entered';
                }}
            }}
            return 'not found';
        }})()
    "#, verification_code);
    
    let eval_msg = json!({
        "id": next_id(),
        "method": "Runtime.evaluate",
        "params": { "expression": enter_code_js }
    });
    write.send(Message::Text(eval_msg.to_string())).await.map_err(|e| e.to_string())?;
    let _ = read.next().await;
    
    sleep(Duration::from_secs(1)).await;
    
    emit_progress(&app, "code_entered", "✅ 验证码已输入", "success");
    
    // 点击验证
    emit_progress(&app, "clicking_verify", "正在点击验证...", "info");
    
    let click_verify_js = r#"
        (function() {
            const buttons = document.querySelectorAll('button, input[type="submit"]');
            for (const btn of buttons) {
                const text = btn.textContent.toLowerCase();
                if (text.includes('verify') || text.includes('confirm') || text.includes('验证')) {
                    btn.click();
                    return 'clicked';
                }
            }
            return 'not found';
        })()
    "#;
    
    let eval_msg = json!({
        "id": next_id(),
        "method": "Runtime.evaluate",
        "params": { "expression": click_verify_js }
    });
    write.send(Message::Text(eval_msg.to_string())).await.map_err(|e| e.to_string())?;
    let _ = read.next().await;
    
    sleep(Duration::from_secs(3)).await;
    if should_stop() { return Ok(()); }
    
    // 输入密码
    emit_progress(&app, "entering_password", "正在输入密码...", "info");
    
    let enter_password_js = format!(r#"
        (function() {{
            const inputs = document.querySelectorAll('input[type="password"]');
            let count = 0;
            for (const input of inputs) {{
                if (input.offsetParent !== null) {{
                    input.focus();
                    input.value = '{}';
                    input.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    input.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    count++;
                }}
            }}
            return count + ' password fields filled';
        }})()
    "#, password);
    
    let eval_msg = json!({
        "id": next_id(),
        "method": "Runtime.evaluate",
        "params": { "expression": enter_password_js }
    });
    write.send(Message::Text(eval_msg.to_string())).await.map_err(|e| e.to_string())?;
    let _ = read.next().await;
    
    sleep(Duration::from_secs(1)).await;
    
    emit_progress(&app, "password_entered", "✅ 密码已输入", "success");
    
    // 点击创建账号
    emit_progress(&app, "creating_account", "正在创建账号...", "info");
    
    let click_create_js = r#"
        (function() {
            const buttons = document.querySelectorAll('button, input[type="submit"]');
            for (const btn of buttons) {
                const text = btn.textContent.toLowerCase();
                if (text.includes('create') || text.includes('submit') || text.includes('注册') || text.includes('continue')) {
                    btn.click();
                    return 'clicked';
                }
            }
            return 'not found';
        })()
    "#;
    
    let eval_msg = json!({
        "id": next_id(),
        "method": "Runtime.evaluate",
        "params": { "expression": click_create_js }
    });
    write.send(Message::Text(eval_msg.to_string())).await.map_err(|e| e.to_string())?;
    let _ = read.next().await;
    
    sleep(Duration::from_secs(5)).await;
    
    emit_progress(&app, "completed", "🎉 注册流程完成！请使用「桌面授权登录」或「网页授权登录」添加账号", "success");
    
    Ok(())
}

#[tauri::command]
pub async fn submit_verification_code(code: String) -> Result<(), String> {
    if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
        return Err("验证码必须是 6 位数字".to_string());
    }
    
    let mut verification_code = VERIFICATION_CODE.lock().unwrap();
    *verification_code = Some(code);
    
    Ok(())
}

#[tauri::command]
pub async fn stop_registration() -> Result<(), String> {
    REGISTRATION_RUNNING.store(false, Ordering::SeqCst);
    
    // 关闭浏览器
    let mut process = BROWSER_PROCESS.lock().unwrap();
    if let Some(mut child) = process.take() {
        let _ = child.kill();
    }
    
    Ok(())
}
