# 测试命令清单

## 1. 构建和运行应用

### 开发模式（推荐用于测试）
```bash
cd 123
npm run tauri dev
```

### 生产构建
```bash
cd 123
npm run tauri build
```

## 2. 测试删除功能

### 步骤:
1. 启动应用（开发模式）
2. 按 F12 打开开发者工具
3. 切换到 Console 标签
4. 导航到"自动注册"页面
5. 点击任意账号的删除按钮（红色垃圾桶图标）
6. 观察控制台输出

### 预期的控制台输出:

#### 成功情况:
```
[Delete] Starting delete for account ID: 1
[Delete] User confirmation: true
[API] Calling auto_register_delete_account with id: 1
[API] Delete result: undefined
[Delete] Delete successful, refreshing list
```

#### 用户取消:
```
[Delete] Starting delete for account ID: 1
[Delete] User confirmation: false
[Delete] User cancelled deletion
```

#### 错误情况:
```
[Delete] Starting delete for account ID: 1
[Delete] User confirmation: true
[API] Calling auto_register_delete_account with id: 1
[API] Delete error: <错误信息>
[Delete] Error during deletion: <错误信息>
```

## 3. 可能的问题和解决方案

### 问题 1: 没有任何日志输出
**原因**: 按钮的 onClick 事件未触发
**解决**: 
- 检查浏览器控制台是否有 JavaScript 错误
- 检查按钮是否被禁用（disabled 属性）
- 尝试刷新页面

### 问题 2: 确认对话框不显示
**原因**: Tauri dialog 插件未正确配置
**解决**: 
- 检查 `123/src-tauri/tauri.conf.json` 中是否包含 dialog 插件
- 检查 `123/src-tauri/Cargo.toml` 中是否有 `tauri-plugin-dialog` 依赖

### 问题 3: API 调用失败
**原因**: 后端命令未注册或数据库问题
**解决**: 
- 检查 Tauri 后端日志（终端输出）
- 确认数据库文件存在且可访问
- 重新构建应用

### 问题 4: 删除成功但列表未刷新
**原因**: `onRefresh` 回调未正确执行
**解决**: 
- 手动刷新页面
- 检查父组件的 `onRefresh` 实现

## 4. 验证所有功能

### 基本功能测试清单:
- [ ] 导入账号
- [ ] 查看账号列表
- [ ] 查看账号详情
- [ ] 编辑账号
- [ ] 删除单个账号
- [ ] 删除所有账号
- [ ] 开始注册
- [ ] 批量注册
- [ ] 导出账号
- [ ] 获取邮件
- [ ] 获取 Kiro 凭证
- [ ] 批量获取凭证

## 5. 调试技巧

### 查看 Tauri 后端日志:
运行 `npm run tauri dev` 时，终端会显示 Rust 后端的日志输出。

### 查看前端日志:
在浏览器开发者工具的 Console 标签中查看。

### 查看数据库:
数据库文件位置（Windows）:
```
%APPDATA%\com.kiro.account-manager-pro\database.db
```

可以使用 SQLite 浏览器工具（如 DB Browser for SQLite）打开查看。

## 6. 常见错误信息

### "Command auto_register_delete_account not found"
**原因**: 命令未注册或名称不匹配
**状态**: ✅ 已修复

### "Database is locked"
**原因**: 多个进程同时访问数据库
**解决**: 关闭所有应用实例，重新启动

### "Failed to initialize database"
**原因**: 数据库文件权限问题或路径不存在
**解决**: 检查应用数据目录权限

## 7. 获取帮助

如果问题仍然存在，请提供:
1. 完整的控制台日志输出
2. Tauri 后端日志输出
3. 具体的错误信息
4. 操作步骤
