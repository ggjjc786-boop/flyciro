# 浏览器自动化登录修复说明

## 问题描述

用户报告点击账号列表中的"导入"按钮后，浏览器会打开 AWS Builder ID 登录页面，但不会自动填写账号密码进行登录获取 token。

## 问题原因

`perform_browser_authorization` 函数使用的是通用的 CSS 选择器来定位页面元素，这些选择器无法准确定位 AWS Builder ID 登录页面的特定元素，导致自动化流程失败。

## 解决方案

### 1. 使用精确的 XPath 选择器

将 `perform_browser_authorization` 函数中的所有 CSS 选择器替换为精确的 XPath 选择器，这些选择器与 AWS Builder ID 登录页面的实际 HTML 结构完全匹配：

- **邮箱输入框**: `/html/body/div/div/main/div/div/form/div[1]/div/awsui-input/div/div[1]/div[1]/div/input`
- **密码输入框**: `/html/body/div/div/main/div/div/form/div[1]/div/awsui-input/div/div[1]/div[1]/div/input`
- **下一步按钮**: `/html/body/div/div/main/div/div/form/div[2]/div/div/awsui-button/button`
- **登录按钮**: `/html/body/div/div/main/div/div/form/div[2]/div/div/awsui-button/button`
- **验证码输入框**: `/html/body/div/div/main/div/div/form/div[1]/div/awsui-input/div/div[1]/div[1]/div/input`
- **验证按钮**: `/html/body/div/div/main/div/div/form/div[2]/div/div/awsui-button/button`
- **允许按钮**: `/html/body/div/div/main/div/div/form/div[2]/span/span/awsui-button/button`

### 2. 删除重复的旧代码

文件中存在重复的 `perform_browser_authorization` 函数实现，删除了使用通用 CSS 选择器的旧版本代码（约 264 行），保留了使用精确 XPath 的新版本。

### 3. 完整的登录流程

修复后的函数实现了完整的 AWS Builder ID 登录流程：

1. 导航到验证 URL
2. 点击"Confirm and continue"按钮（如果存在）
3. 输入邮箱地址
4. 点击下一步
5. 输入密码
6. 点击登录
7. 如果需要验证码，从邮箱获取并输入
8. 点击允许/授权按钮
9. 等待授权完成

## 修改的文件

- `123/src-tauri/src/commands/auto_register_cmd.rs`
  - 修改了 `perform_browser_authorization` 函数
  - 删除了重复的旧代码（约 264 行）
  - 使用精确的 XPath 选择器替代通用 CSS 选择器

## 测试建议

1. 在账号列表中找到状态为"已注册"的账号
2. 点击该账号行的绿色"导入"按钮（Upload 图标）
3. 观察浏览器是否自动：
   - 打开 AWS Builder ID 登录页面
   - 自动填写邮箱
   - 自动填写密码
   - 自动点击登录
   - 如需验证码，自动获取并填写
   - 自动点击允许按钮
4. 确认账号成功导入到主账号池

## 相关功能

- **单个账号导入**: `auto_register_get_credentials_and_import` 命令
- **批量导入**: `auto_register_import_to_main` 命令
- **注册后自动获取凭证**: 在 `auto_register_start_registration` 中自动调用 `perform_kiro_login`

## 提交信息

- Commit: 修复浏览器自动化登录：使用精确的 XPath 选择器替代通用 CSS 选择器，删除重复的旧代码
- 删除行数: 327 行
- 新增行数: 63 行
- 净减少: 264 行
