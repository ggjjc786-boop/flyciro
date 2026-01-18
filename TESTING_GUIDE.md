# 测试指南 - 自动导入功能

## 前提条件

由于你的系统缺少 Windows SDK 的 RC.EXE，无法运行 `npm run tauri dev`。但代码已经完成并通过了语法检查。

## 功能测试步骤（当你解决 RC.EXE 问题后）

### 1. 准备测试数据

1. 启动应用
2. 进入"自动注册"页面
3. 导入或添加一些测试账号（需要有效的邮箱账号信息）

### 2. 测试注册流程

1. 选择一个账号，点击"注册"按钮
2. 观察注册过程：
   - 状态应该变为"进行中"
   - 浏览器会自动打开并完成注册流程
   - 注册成功后状态变为"已注册"
   - 系统会自动获取 AWS Builder ID 凭证
3. 检查账号详情，确认已获取到：
   - kiro_client_id
   - kiro_client_secret
   - kiro_refresh_token
   - kiro_access_token
   - kiro_id_token

### 3. 测试导入功能

1. 在自动注册页面，点击"导入到主账号列表"按钮
2. 确认导入操作
3. 等待导入完成，应该显示成功消息
4. 切换到主账号管理页面
5. 验证导入的账号：
   - 账号应该出现在列表中
   - 标签应该是"Kiro BuilderId 账号 (自动注册)"
   - Provider 应该是"BuilderId"
   - 应该有完整的凭证信息
   - 应该有 usage 数据

### 4. 测试重复导入

1. 再次点击"导入到主账号列表"
2. 确认操作
3. 验证不会创建重复账号，而是更新现有账号

### 5. 测试批量场景

1. 注册多个账号
2. 一次性导入所有账号
3. 检查导入结果统计是否正确

## 预期结果

### 成功场景

- 注册完成后自动获取凭证
- 导入成功，显示"导入完成！成功: X, 失败: 0"
- 主账号列表中出现导入的账号
- 账号可以正常使用（刷新 token、获取 usage 等）

### 失败场景

- 如果账号没有凭证，会提示"没有可导入的账号（需要先获取凭证）"
- 如果凭证无效，该账号会被跳过，显示在失败计数中
- 控制台会输出详细的错误信息

## 调试技巧

### 查看日志

在开发模式下，打开浏览器开发者工具（F12）查看：

1. Console 标签：查看前端日志
2. Network 标签：查看 API 调用

在 Rust 后端，查看终端输出：

```
[Auto Register] Registration successful, now fetching Kiro credentials...
[Auto Register] Kiro credentials obtained successfully!
[Import] Successfully imported account: xxx@example.com
```

### 常见问题

1. **导入失败：token 无效**
   - 原因：凭证已过期
   - 解决：重新获取凭证或重新注册

2. **导入失败：网络错误**
   - 原因：无法连接到 AWS API
   - 解决：检查网络连接和代理设置

3. **账号重复**
   - 原因：邮箱和 provider 相同
   - 行为：会更新现有账号而不是创建新账号

## 手动验证数据

### 检查自动注册数据库

数据库位置：`%APPDATA%/.kiro-account-manager/auto_register.db`

使用 SQLite 工具查询：

```sql
SELECT email, status, kiro_client_id, kiro_refresh_token 
FROM accounts 
WHERE status = 'registered';
```

### 检查主账号数据库

数据库位置：`%APPDATA%/.kiro-account-manager/accounts.json`

打开 JSON 文件查看导入的账号。

## 性能测试

1. 导入 10 个账号，记录耗时
2. 导入 50 个账号，记录耗时
3. 检查内存使用情况

## 安全测试

1. 验证凭证是否正确加密存储
2. 验证导入过程中不会泄露敏感信息
3. 验证错误消息不会暴露凭证

## 回归测试

确保新功能不影响现有功能：

1. 主账号管理的所有功能正常
2. 自动注册的其他功能正常
3. 账号刷新功能正常
4. 账号删除功能正常
