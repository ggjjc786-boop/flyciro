# Kiro 自动注册功能迁移指南

## 概述

本指南说明如何将 `kiro自动注册源代码` 文件夹中的自动注册功能迁移到 123 项目中。

## 功能说明

自动注册功能的完整流程：

1. **输入微软邮箱信息**
   - 邮箱地址 (email)
   - 邮箱密码 (email_password)  
   - Microsoft OAuth Client ID (client_id)
   - Microsoft OAuth Refresh Token (refresh_token)

2. **自动注册 Kiro 账号**
   - 使用浏览器自动化访问 https://app.kiro.dev/signin
   - 点击 Google 登录按钮
   - 输入邮箱地址
   - 输入姓名
   - 从微软邮箱获取验证码并自动填入
   - 设置 Kiro 密码
   - 完成注册

3. **获取 Kiro 凭证**
   - 使用 AWS SSO OIDC 设备授权流程
   - 在浏览器中完成授权
   - 获取 client_id, client_secret, access_token, refresh_token

## 需要迁移的文件

### Rust 后端文件 (src-tauri/src/)

```
kiro自动注册源代码/src-tauri/src/
├── models.rs              -> 123/src-tauri/src/auto_register/models.rs
├── database.rs            -> 123/src-tauri/src/auto_register/database.rs
├── browser_automation.rs  -> 123/src-tauri/src/auto_register/browser_automation.rs
├── graph_api.rs           -> 123/src-tauri/src/auto_register/graph_api.rs
├── aws_sso_client.rs      -> 123/src-tauri/src/auto_register/aws_sso_client.rs
└── commands.rs            -> 123/src-tauri/src/commands/auto_register_cmd.rs
```

### 前端组件文件 (src/components/)

```
kiro自动注册源代码/src/components/
├── AccountsTable.tsx      -> 123/src/components/AutoRegister/AccountsTable.tsx
├── AccountsTable.css      -> 123/src/components/AutoRegister/AccountsTable.css
├── ControlPanel.tsx       -> 123/src/components/AutoRegister/ControlPanel.tsx
├── ControlPanel.css       -> 123/src/components/AutoRegister/ControlPanel.css
├── ImportPanel.tsx        -> 123/src/components/AutoRegister/ImportPanel.tsx
├── ImportPanel.css        -> 123/src/components/AutoRegister/ImportPanel.css
├── TitleBar.tsx           -> 123/src/components/AutoRegister/TitleBar.tsx
└── TitleBar.css           -> 123/src/components/AutoRegister/TitleBar.css
```

### 其他文件

```
kiro自动注册源代码/src/
├── api.ts                 -> 123/src/api/autoRegister.ts
├── store.ts               -> 123/src/stores/autoRegisterStore.ts
└── App.tsx                -> 参考集成到 123/src/App.tsx
```

## 迁移步骤

### 1. 复制 Rust 后端文件

```powershell
# 在项目根目录执行
cd 123

# 创建 auto_register 模块目录
mkdir src-tauri\src\auto_register

# 复制文件
copy ..\kiro自动注册源代码\src-tauri\src\models.rs src-tauri\src\auto_register\
copy ..\kiro自动注册源代码\src-tauri\src\database.rs src-tauri\src\auto_register\
copy ..\kiro自动注册源代码\src-tauri\src\browser_automation.rs src-tauri\src\auto_register\
copy ..\kiro自动注册源代码\src-tauri\src\graph_api.rs src-tauri\src\auto_register\
copy ..\kiro自动注册源代码\src-tauri\src\aws_sso_client.rs src-tauri\src\auto_register\
copy ..\kiro自动注册源代码\src-tauri\src\commands.rs src-tauri\src\commands\auto_register_cmd.rs
```

### 2. 创建模块文件

创建 `123/src-tauri/src/auto_register/mod.rs`:

```rust
pub mod models;
pub mod database;
pub mod browser_automation;
pub mod graph_api;
pub mod aws_sso_client;

pub use models::*;
pub use database::*;
pub use browser_automation::*;
pub use graph_api::*;
pub use aws_sso_client::*;
```

### 3. 更新 lib.rs

在 `123/src-tauri/src/lib.rs` 中添加：

```rust
mod auto_register;
```

### 4. 更新 commands/mod.rs

在 `123/src-tauri/src/commands/mod.rs` 中添加：

```rust
pub mod auto_register_cmd;
```

### 5. 更新 main.rs

在 `123/src-tauri/src/main.rs` 中：

1. 添加导入：
```rust
use commands::auto_register_cmd::*;
```

2. 在 setup 中初始化数据库：
```rust
.setup(|app| {
    // 初始化自动注册数据库
    tauri::async_runtime::block_on(async {
        auto_register::database::init_database(app.handle()).await
            .expect("Failed to initialize auto register database");
    });
    
    // ... 其他初始化代码
})
```

3. 注册命令：
```rust
.invoke_handler(tauri::generate_handler![
    // ... 现有命令
    
    // 自动注册命令
    get_accounts,
    add_account,
    update_account,
    delete_account,
    delete_all_accounts,
    import_accounts,
    get_settings,
    update_settings,
    start_registration,
    start_batch_registration,
    export_accounts,
    fetch_latest_email,
    get_kiro_credentials,
    batch_fetch_kiro_credentials,
])
```

### 6. 更新 Cargo.toml

在 `123/src-tauri/Cargo.toml` 中添加依赖：

```toml
[dependencies]
# ... 现有依赖

# 自动注册功能依赖
headless_chrome = "1.0"
regex = "1.10"
```

### 7. 复制前端文件

```powershell
# 创建目录
mkdir src\components\AutoRegister
mkdir src\api
mkdir src\stores

# 复制组件
copy ..\kiro自动注册源代码\src\components\*.tsx src\components\AutoRegister\
copy ..\kiro自动注册源代码\src\components\*.css src\components\AutoRegister\

# 复制 API 和 Store
copy ..\kiro自动注册源代码\src\api.ts src\api\autoRegister.ts
copy ..\kiro自动注册源代码\src\store.ts src\stores\autoRegisterStore.ts
```

### 8. 集成到 App.tsx

在 `123/src/App.tsx` 中添加路由：

```typescript
import { AutoRegisterMain } from './components/AutoRegister/Main';

// 在 renderContent 中添加
case 'auto-register': return <AutoRegisterMain />
```

### 9. 添加国际化文本

在 `123/locales/zh-CN.json` 中添加：

```json
{
  "autoRegister.title": "自动注册",
  "autoRegister.import": "导入账号",
  "autoRegister.startRegistration": "开始注册",
  "autoRegister.batchRegistration": "批量注册",
  "autoRegister.getCredentials": "获取凭证",
  "autoRegister.email": "邮箱",
  "autoRegister.password": "密码",
  "autoRegister.clientId": "Client ID",
  "autoRegister.refreshToken": "Refresh Token",
  "autoRegister.status": "状态",
  "autoRegister.kiroPassword": "Kiro 密码",
  "autoRegister.notRegistered": "未注册",
  "autoRegister.inProgress": "注册中",
  "autoRegister.registered": "已注册",
  "autoRegister.error": "错误"
}
```

## 测试

1. 编译项目：
```powershell
cd 123
npm run tauri build
```

2. 测试功能：
   - 导入微软邮箱账号信息
   - 点击"开始注册"测试单个账号注册
   - 点击"批量注册"测试批量注册
   - 点击"获取凭证"测试获取 Kiro 凭证

## 注意事项

1. 需要安装 Chrome/Chromium 浏览器
2. 微软邮箱需要开启 OAuth 并获取 client_id 和 refresh_token
3. 注册过程需要访问外网
4. 浏览器自动化可能因为页面变化而失败，需要更新选择器

## 故障排除

如果遇到编译错误：

1. 检查所有文件路径是否正确
2. 检查 Cargo.toml 依赖是否完整
3. 运行 `cargo clean` 清理缓存
4. 检查 Rust 版本是否 >= 1.70

如果遇到运行时错误：

1. 检查数据库是否正确初始化
2. 检查浏览器是否正确启动
3. 查看控制台日志获取详细错误信息
