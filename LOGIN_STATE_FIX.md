# 登录状态显示问题修复

## 问题描述
用户报告："明明登录账号了主页却显示没有登录"

自动注册完成后，账号成功导入到主账号列表，但前端主页仍然显示未登录状态。

## 问题原因
1. **缺少事件通知**：账号导入成功后，没有发送 `login-success` 事件通知前端刷新
2. **auth 状态未更新**：账号虽然添加到账号列表，但全局的 `auth.user` 状态没有更新，导致前端认为用户未登录

## 解决方案

### 1. 添加 AppHandle 参数
修改以下命令函数签名，添加 `app_handle: tauri::AppHandle` 参数：
- `auto_register_start_registration`
- `auto_register_get_credentials_and_import`

### 2. 更新 auth 状态
在账号成功导入后，更新全局 auth 状态：
```rust
// 更新 auth 状态，标记用户已登录
let user = crate::commands::auth_cmd::User {
    id: uuid::Uuid::new_v4().to_string(),
    email: account.email.clone(),
    name: account.email.split('@').next().unwrap_or("User").to_string(),
    avatar: None,
    provider: "BuilderId".to_string(),
};
*app_state.auth.user.lock().unwrap() = Some(user);
*app_state.auth.access_token.lock().unwrap() = Some(auth_result.access_token.clone());
*app_state.auth.refresh_token.lock().unwrap() = Some(auth_result.refresh_token.clone());
```

### 3. 发送登录成功事件
在更新 auth 状态后，发送事件通知前端：
```rust
// 发送登录成功事件，通知前端刷新
if !account_id_for_event.is_empty() {
    let _ = app_handle.emit("login-success", account_id_for_event);
    println!("[Auto Register] Emitted login-success event");
}
```

## 修改的文件
- `123/src-tauri/src/commands/auto_register_cmd.rs`
  - `auto_register_start_registration` 函数
  - `auto_register_get_credentials_and_import` 函数

## 工作流程
1. 用户点击"开始注册"
2. 自动完成 Kiro 注册
3. 浏览器导航到 AWS SSO 授权页面
4. 用户手动点击授权按钮
5. 程序检测授权完成，自动获取 token
6. 使用 IdcProvider 刷新 token，获取 usage 数据
7. 导入到主账号列表
8. **更新 auth 状态（新增）**
9. **发送 login-success 事件（新增）**
10. 前端接收事件，刷新账号列表和登录状态
11. 主页显示已登录状态

## 前端事件监听
前端已经在 `App.tsx` 中监听 `login-success` 事件：
```typescript
const unlisten = listen('login-success', (event) => {
  console.log('Login success in App:', event.payload)
  checkAuth()  // 刷新登录状态
  setActiveMenu('token')  // 切换到账号管理页面
})
```

前端也在 `useAccounts.js` 中监听该事件：
```javascript
const unlistenLoginSuccess = listen('login-success', () => loadAccounts())
```

## 测试建议
1. 启动自动注册流程
2. 完成注册和授权
3. 检查主页是否显示已登录状态
4. 检查账号列表是否包含新注册的账号
5. 检查控制台日志确认事件已发送

## 注意事项
- 确保在释放 `store` 锁之后再更新 `auth` 状态，避免死锁
- 使用 `drop(store)` 显式释放锁
- 事件发送使用 `let _ = app_handle.emit(...)` 忽略错误，避免因事件发送失败导致整个流程失败
