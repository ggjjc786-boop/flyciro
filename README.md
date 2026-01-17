# Kiro Account Manager Pro

完整的 Kiro IDE 账号管理系统，集成了账号管理、自动注册、Token 管理等全部功能。

## 功能特性

### 账号管理
- 账号列表展示（卡片/列表视图）
- 一键切换账号
- 批量刷新 Token
- 批量操作（删除、导入、导出）
- 标签管理
- 高级筛选和排序

### 登录方式
- Desktop OAuth（Google/GitHub/BuilderId）
- Web OAuth
- SSO Token 导入

### Kiro 配置
- MCP 服务器管理
- Powers 管理
- Steering 规则管理

### 系统设置
- 4 种主题（浅色/深色/紫色/绿色）
- AI 模型选择
- 代理配置
- 浏览器设置
- 机器码管理

### 自动化功能
- Token 自动刷新
- 自动更新检查
- IDE 集成

## 快速开始

```bash
# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 构建
npm run tauri build
```

## 技术栈

- **前端**: React 18 + TypeScript + Tailwind CSS
- **后端**: Rust + Tauri 2.x
- **数据库**: SQLite
- **状态管理**: Context API
- **国际化**: i18next

## 许可证

CC BY-NC-SA 4.0 - 禁止商业使用
