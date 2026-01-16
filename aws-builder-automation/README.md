# AWS Builder ID 自动注册系统

一个基于 Tauri 2.x + React 18 + Rust 开发的自动化注册系统，用于批量注册 AWS Builder ID 账号。

## 技术栈

### 后端
- **框架**: Tauri 2.x
- **语言**: Rust
- **数据库**: SQLite3
- **主要依赖**:
  - `rusqlite` - SQLite数据库操作
  - `reqwest` - HTTP客户端(用于Microsoft Graph API)
  - `headless_chrome` - 浏览器自动化
  - `tokio` - 异步运行时
  - `serde` - 序列化/反序列化
  - `chrono` - 日期时间处理
  - `rand` - 随机数生成

### 前端
- **UI框架**: React 18
- **构建工具**: Vite 5
- **状态管理**: Zustand
- **UI组件**: Lucide React (图标)
- **样式**: 原生CSS + CSS变量 (支持亮色/暗黑主题)

## 核心功能

### 1. 账号管理模块
- **数据表格展示**:
  - 序号、注册邮箱、邮箱密码
  - 客户端ID(隐藏)、refresh_token令牌(隐藏)
  - Kiro密码、状态(未注册/进行中/已注册/异常)
  - 异常原因、操作按钮
- **分页**: 每页20条记录
- **排序**: 支持所有字段排序
- **搜索**: 支持邮箱和状态搜索
- **操作**: 查看详情、编辑、开始注册、删除

### 2. 数据导入功能
支持两种导入方式:
- **富文本粘贴导入**: 直接粘贴数据到文本框
- **TXT文件导入**: 选择TXT文件导入

**数据格式**:
```
邮箱地址----邮箱密码----客户端ID----refresh_token令牌
```

**示例**:
```
user@example.com----password123----client-id-here----refresh-token-here
```

**验证机制**:
- 格式验证(必须包含4个字段)
- 邮箱格式验证
- 字段非空验证
- 导入结果统计(成功数、失败数、错误详情)

### 3. 数据管理功能
- **状态筛选**: 支持按状态筛选账号(全部/未注册/进行中/已注册/异常)
- **删除全部**: 双重确认机制防止误操作
- **批量操作**: 支持批量导入和管理

### 4. 系统设置模块
- **浏览器运行模式**:
  - 后台运行: 浏览器窗口不可见
  - 前台运行: 浏览器窗口可见，可实时观察注册过程
- **设置持久化**: 重启应用后保持用户偏好

### 5. 自动注册功能

**浏览器隐私保护配置**:
- ✅ 彻底清除浏览器缓存(Cookie、本地存储、会话存储)
- ✅ 无痕浏览模式
- ✅ 指纹随机化:
  - 操作系统: Windows 10/11 随机选择
  - 窗口尺寸: 800-1920x600-1080 随机
  - Canvas指纹随机化
  - WebGL指纹随机化
  - AudioContext指纹随机化
  - ClientRects随机化
  - 媒体设备列表随机化
  - Speech Voices中文语音列表
- ✅ WebRTC IP保护
- ✅ 端口扫描保护
- ✅ 证书错误忽略

**注册流程**:
1. 访问 https://app.kiro.dev/signin
2. 点击 Google 登录按钮
3. 输入邮箱地址
4. 输入姓名(随机中文姓名)
5. 等待邮箱验证码(通过Microsoft Graph API自动获取)
6. 输入验证码(自动提取6位数字)
7. 设置密码(随机生成16位安全密码)
8. 完成注册并保存密码

**邮件验证码获取**:
- 使用Microsoft Graph API
- 支持refresh_token自动刷新access_token
- 自动提取验证码(智能识别6位数字)
- 60秒超时自动重发
- 错误处理和重试机制

### 6. 现代化UI设计

**2025年现代设计标准**:
- ✨ 精致的线型图标系统(Lucide React)
- 🎨 亮色/暗黑主题无缝切换
- 📱 响应式设计(支持多种屏幕尺寸)
- 🎭 流畅的动画过渡效果
- 🎯 隐藏式标题栏(向上滚动隐藏,向下滚动显示)
- 💫 高端视觉效果(阴影、圆角、渐变)
- ⚡ 优秀的交互反馈(加载状态、错误提示、成功提示)

**配色系统**:
- 主色调: 蓝色 (#4c6ef5)
- 成功: 绿色 (#51cf66)
- 警告: 黄色 (#ffd43b)
- 错误: 红色 (#ff6b6b)
- 信息: 蓝色 (#4dabf7)

## 项目结构

```
aws-builder-automation/
├── src/                          # 前端源码
│   ├── components/               # React组件
│   │   ├── TitleBar.tsx         # 标题栏组件
│   │   ├── AccountsTable.tsx    # 账号表格组件
│   │   ├── ImportPanel.tsx      # 导入面板组件
│   │   └── ControlPanel.tsx     # 控制面板组件
│   ├── App.tsx                  # 主应用组件
│   ├── store.ts                 # Zustand状态管理
│   ├── api.ts                   # Tauri API调用封装
│   └── index.css                # 全局样式
│
├── src-tauri/                    # Tauri后端
│   ├── src/
│   │   ├── lib.rs               # 主入口
│   │   ├── models.rs            # 数据模型
│   │   ├── database.rs          # SQLite数据库操作
│   │   ├── graph_api.rs         # Microsoft Graph API
│   │   ├── browser_automation.rs # 浏览器自动化
│   │   └── commands.rs          # Tauri命令
│   ├── Cargo.toml               # Rust依赖配置
│   └── tauri.conf.json          # Tauri配置
│
└── README.md                     # 项目文档
```

## 安装与使用

### 环境要求
- Node.js >= 18
- Rust >= 1.70
- Chrome/Chromium 浏览器

### 安装依赖
```bash
cd aws-builder-automation
npm install
```

### 开发模式
```bash
npm run tauri dev
```

### 构建应用
```bash
npm run tauri build
```

构建完成后，可执行文件位于 `src-tauri/target/release/`

## 使用说明

### 1. 准备账号数据
按照以下格式准备账号数据:
```
email@domain.com----password123----client_id----refresh_token
```

### 2. 导入账号
- 方式一: 将数据粘贴到导入面板的文本框，点击"从文本导入"
- 方式二: 将数据保存为TXT文件，点击"从文件导入"选择文件

### 3. 配置系统设置
- 点击"系统设置"按钮
- 选择浏览器运行模式(后台/前台)
- 点击"保存设置"

### 4. 开始注册
- 在账号列表中找到状态为"未注册"的账号
- 点击"开始注册"按钮(播放图标)
- 系统自动执行注册流程
- 注册完成后状态更新为"已注册"，密码自动保存

### 5. 查看结果
- 查看账号状态和Kiro密码
- 使用状态筛选查看不同状态的账号
- 导出或使用注册成功的账号

## 数据库结构

### accounts 表
```sql
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    email_password TEXT NOT NULL,
    client_id TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    kiro_password TEXT,
    status TEXT NOT NULL DEFAULT 'not_registered',
    error_reason TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

### settings 表
```sql
CREATE TABLE settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    browser_mode TEXT NOT NULL DEFAULT 'background'
);
```

## 安全性说明

### 数据安全
- 所有数据存储在本地SQLite数据库
- 不与任何第三方服务器通信(除Microsoft Graph API和目标网站)
- refresh_token和client_id在界面上隐藏

### 隐私保护
- 浏览器指纹随机化防止追踪
- 无痕浏览模式不留痕迹
- WebRTC IP保护防止真实IP泄露
- 端口扫描保护防止本地信息泄露

### 访问控制
- Microsoft Graph API使用OAuth 2.0认证
- refresh_token安全存储和使用
- 不记录任何用户操作日志

## 故障排除

### 编译错误
1. 确保Rust工具链已正确安装: `rustc --version`
2. 更新Rust: `rustup update`
3. 清理缓存: `cargo clean`

### 运行时错误
1. 检查Chrome/Chromium是否已安装
2. 确保网络连接正常
3. 检查Microsoft Graph API凭证是否有效

### 注册失败
1. 检查邮箱和refresh_token是否有效
2. 确认Microsoft Graph API权限配置正确
3. 检查目标网站是否可访问
4. 查看错误原因字段获取详细信息

## 注意事项

1. **合法使用**: 本工具仅供学习和研究使用，请遵守相关法律法规和网站服务条款
2. **频率限制**: 建议合理控制注册频率，避免被目标网站限制
3. **数据备份**: 重要数据请及时备份
4. **安全存储**: 妥善保管生成的密码和账号信息

## 开发说明

### 代码规范
- Rust代码遵循官方风格指南
- React组件使用函数式组件和Hooks
- TypeScript严格模式
- 使用ESLint和Prettier格式化代码

### 测试
```bash
# Rust测试
cd src-tauri
cargo test

# 前端测试
npm test
```

### 贡献指南
1. Fork项目
2. 创建特性分支
3. 提交变更
4. 推送到分支
5. 创建Pull Request

## 许可证

本项目仅供学习交流使用。

## 致谢

- [Tauri](https://tauri.app/) - 跨平台桌面应用框架
- [React](https://react.dev/) - UI框架
- [headless_chrome](https://github.com/rust-headless-chrome/rust-headless-chrome) - Rust浏览器自动化库
- [Lucide](https://lucide.dev/) - 图标库

## 更新日志

### v1.0.0 (2025-12-12)
- ✨ 初始版本发布
- ✅ 完整的账号管理功能
- ✅ 数据导入导出
- ✅ 自动注册流程
- ✅ Microsoft Graph API邮件获取
- ✅ 浏览器指纹保护
- ✅ 现代化UI设计
- ✅ 亮色/暗黑主题切换
- ✅ 隐藏式标题栏
