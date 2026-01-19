# 自动授权功能实现

## 功能说明

在自动注册完成后，系统会自动完成 AWS SSO 授权流程，无需用户手动点击授权按钮。

## 实现细节

### 修改的文件
- `123/src-tauri/src/commands/auto_register_cmd.rs`

### 修改内容

在 `perform_kiro_login_with_browser` 函数中添加了自动点击授权按钮的逻辑：

#### Step 4: 第一个页面 - 自动点击 "Confirm and continue" 按钮

第一个页面显示 "Authorization requested" 和验证码，需要点击 "Confirm and continue" 按钮。

使用多个 XPath 选择器尝试定位并点击：

```rust
let confirm_continue_selectors = vec![
    "//button[contains(text(), 'Confirm and continue')]",
    "//button[contains(., 'Confirm and continue')]",
    "//button[contains(text(), '确认并继续')]",
    "//button[contains(., '确认并继续')]",
    "//button[@type='submit' and not(contains(., 'Cancel'))]",
    "//button[@type='submit' and not(contains(., '取消'))]",
];
```

#### Step 5: 第二个页面 - 自动点击 "Allow access" 按钮

第二个页面显示 "Allow Kiro Account Manager to access your data?"，需要点击 "Allow access" 按钮。

使用多个 XPath 选择器尝试定位并点击：

```rust
let allow_access_selectors = vec![
    "//button[contains(text(), 'Allow access')]",
    "//button[contains(., 'Allow access')]",
    "//button[@type='submit' and contains(., 'Allow')]",
    "//*[contains(@class, 'awsui-button') and contains(., 'Allow access')]",
    "//button[contains(@class, 'awsui-button-variant-primary')]",
];
```

### 工作流程

1. 注册完成后，浏览器导航到 AWS SSO 授权页面
2. **第一个页面**：
   - 等待 "Confirm and continue" 按钮出现（最多等待 10 秒）
   - 自动点击 "Confirm and continue" 按钮
   - 等待 4 秒让页面加载
3. **第二个页面**：
   - 等待 "Allow access" 按钮出现（最多等待 10 秒）
   - 自动点击 "Allow access" 按钮
   - 等待 2 秒
4. 开始轮询获取 Token

### 容错处理

- 如果找不到按钮，会尝试多个不同的选择器
- 如果所有选择器都失败，会打印警告信息但继续执行
- 每个步骤都有详细的日志输出，便于调试

### 日志输出

```
[Kiro Login] Step 4: Auto-clicking 'Confirm and continue' button on first page...
[Kiro Login] Found 'Confirm and continue' button with selector: ...
[Kiro Login] Successfully clicked 'Confirm and continue' button
[Kiro Login] Step 5: Auto-clicking 'Allow access' button on second page...
[Kiro Login] Found 'Allow access' button with selector: ...
[Kiro Login] Successfully clicked 'Allow access' button
[Kiro Login] Authorization buttons clicked, waiting for token...
```

## 授权页面顺序

1. **第一个页面**：Authorization requested
   - 显示验证码（如 NZZD-LTVL）
   - 按钮：Confirm and continue（橙色）/ Cancel（蓝色）

2. **第二个页面**：Allow Kiro Account Manager to access your data?
   - 显示权限请求
   - 按钮：Allow access（橙色）/ Deny access（蓝色）

## 测试建议

1. 运行自动注册流程
2. 观察控制台日志，确认按钮被正确点击
3. 如果按钮没有被点击，检查日志中的警告信息
4. 如果需要，可以根据实际的 HTML 结构调整选择器

## 注意事项

- 此功能依赖于 AWS SSO 页面的 HTML 结构
- 如果 AWS 更新了页面结构，可能需要更新选择器
- 使用了多个备用选择器以提高兼容性
- 两个页面之间有 4 秒的等待时间，确保页面完全加载

