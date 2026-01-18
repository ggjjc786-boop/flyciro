# 自动导入功能完成说明

## 功能概述

实现了自动注册完成后，将 AWS Builder ID 账号自动导入到主账号列表的功能。

## 实现流程

### 1. 注册时自动获取凭证

在 `auto_register_start_registration` 命令中，注册成功后会自动调用 `perform_kiro_login` 获取 AWS Builder ID 凭证：

- Client ID
- Client Secret  
- Refresh Token
- Access Token
- ID Token

这些凭证会自动保存到自动注册数据库的账号记录中。

### 2. 导入到主账号列表

新增 `auto_register_import_to_main` 命令，功能：

- 从自动注册数据库获取所有已注册且有凭证的账号
- 使用主项目的 `IdcProvider` 刷新 token 验证凭证有效性
- 使用 `CodeWhispererClient` 获取账号的 usage 数据
- 将账号添加到主账号列表（`AppState.store.accounts`）
- 如果账号已存在则更新，不存在则新增
- 自动保存到主账号数据库

### 3. 前端 UI

在自动注册控制面板添加了"导入到主账号列表"按钮：

- 绿色按钮，带 Upload 图标
- 点击后弹出确认对话框
- 显示导入结果（成功/失败数量）
- 导入完成后自动刷新账号列表

## 文件修改清单

### 后端 (Rust)

1. **123/src-tauri/src/commands/auto_register_cmd.rs**
   - 完成 `auto_register_import_to_main` 命令实现
   - 修复了原有的错误（使用正确的 Mutex 和保存方法）

2. **123/src-tauri/src/lib.rs**
   - 注册 `auto_register_import_to_main` 命令

### 前端 (TypeScript/React)

1. **123/src/api/autoRegister.ts**
   - 添加 `importToMain()` API 函数
   - 添加 `getKiroCredentials()` 和 `batchFetchKiroCredentials()` API 函数

2. **123/src/components/AutoRegister/ControlPanel.tsx**
   - 添加 `handleImportToMain` 处理函数
   - 添加"导入到主账号列表"按钮
   - 导入 `Upload` 图标

## 使用方法

1. 在自动注册页面添加账号并完成注册
2. 注册成功后，系统会自动获取 AWS Builder ID 凭证
3. 点击"导入到主账号列表"按钮
4. 确认导入操作
5. 系统会将所有已注册且有凭证的账号导入到主账号列表
6. 导入完成后可以在主账号管理页面看到这些账号

## 技术细节

### 凭证获取流程

1. 使用 AWS SSO Client 注册设备客户端
2. 发起设备授权请求
3. 启动浏览器自动完成授权（输入邮箱、密码、验证码等）
4. 轮询获取 Token
5. 保存所有凭证到数据库

### 导入流程

1. 查询所有状态为 "registered" 且有凭证的账号
2. 对每个账号：
   - 使用 IdcProvider 刷新 token
   - 使用 CodeWhispererClient 获取 usage 数据
   - 计算 client_id_hash
   - 创建或更新主账号记录
   - 保存到主账号数据库
3. 返回成功/失败统计

## 注意事项

- 只有状态为 "registered" 且已获取凭证的账号才能导入
- 导入时会验证凭证有效性，无效的账号会跳过
- 如果主账号列表中已存在相同邮箱的 BuilderId 账号，会更新而不是重复添加
- 导入过程中会自动获取最新的 usage 数据和 token

## 后续优化建议

1. 可以添加批量获取凭证功能（已有 `auto_register_batch_fetch_kiro_credentials` 命令）
2. 可以在账号列表中显示是否已获取凭证的状态
3. 可以添加定时自动导入功能
4. 可以添加导入历史记录
