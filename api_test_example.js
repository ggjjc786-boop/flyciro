/**
 * 闲鱼专属 API 调用示例
 * 
 * 本文件提供了调用闲鱼专属 API 的完整示例代码
 * 适用于外部软件（如 Windows 桌面应用）集成
 */

const API_BASE_URL = 'http://localhost:3000';

// ============================================
// 1. 提交账号邮箱信息
// ============================================

/**
 * 提交账号的 Outlook 邮箱信息到服务器
 * @param {Object} accountData 账号数据
 * @returns {Promise<Object>} 提交结果
 */
async function submitAccountEmailInfo(accountData) {
    const {
        username,           // 账号用户名
        password,           // 账号密码
        outlook_email,      // Outlook 邮箱地址
        mail_client_id,     // OAuth Client ID
        mail_refresh_token  // OAuth Refresh Token
    } = accountData;

    try {
        const response = await fetch(`${API_BASE_URL}/api/xianyu/submit-email-info`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                username,
                password,
                outlook_email,
                mail_client_id,
                mail_refresh_token
            })
        });

        const result = await response.json();

        if (result.success) {
            console.log('✅ 邮箱信息提交成功');
            console.log('账号ID:', result.account_id);
            return { success: true, accountId: result.account_id };
        } else {
            console.error('❌ 提交失败:', result.message);
            return { success: false, error: result.message };
        }
    } catch (error) {
        console.error('❌ 网络错误:', error);
        return { success: false, error: error.message };
    }
}

// ============================================
// 2. 批量提交多个账号的邮箱信息
// ============================================

/**
 * 批量提交多个账号的邮箱信息
 * @param {Array<Object>} accounts 账号数组
 */
async function batchSubmitAccounts(accounts) {
    console.log(`开始批量提交 ${accounts.length} 个账号的邮箱信息...`);

    const results = {
        success: 0,
        failed: 0,
        errors: []
    };

    for (let i = 0; i < accounts.length; i++) {
        const account = accounts[i];
        console.log(`\n[${i + 1}/${accounts.length}] 提交账号: ${account.username}`);

        const result = await submitAccountEmailInfo(account);

        if (result.success) {
            results.success++;
        } else {
            results.failed++;
            results.errors.push({
                username: account.username,
                error: result.error
            });
        }

        // 避免请求过快，延迟 500ms
        await new Promise(resolve => setTimeout(resolve, 500));
    }

    console.log('\n============ 批量提交完成 ============');
    console.log(`✅ 成功: ${results.success}`);
    console.log(`❌ 失败: ${results.failed}`);

    if (results.errors.length > 0) {
        console.log('\n失败详情:');
        results.errors.forEach(err => {
            console.log(`  - ${err.username}: ${err.error}`);
        });
    }

    return results;
}

// ============================================
// 3. 测试获取邮箱验证码（可选）
// ============================================

/**
 * 根据账号用户名获取账号ID
 * @param {string} username 账号用户名
 * @returns {Promise<number|null>} 账号ID
 */
async function getAccountId(username) {
    try {
        const response = await fetch(`${API_BASE_URL}/api/xianyu/get-account-id`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ username })
        });

        const result = await response.json();
        return result.success ? result.account_id : null;
    } catch (error) {
        console.error('获取账号ID失败:', error);
        return null;
    }
}

/**
 * 获取账号的邮箱验证码
 * @param {number} accountId 账号ID
 * @returns {Promise<string|null>} 验证码
 */
async function getEmailVerificationCode(accountId) {
    try {
        const response = await fetch(`${API_BASE_URL}/api/xianyu/get-email-code`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ account_id: accountId })
        });

        const result = await response.json();

        if (result.success) {
            console.log('✅ 验证码获取成功:', result.code);
            console.log('邮箱:', result.email);
            return result.code;
        } else {
            console.error('❌ 获取失败:', result.message);
            return null;
        }
    } catch (error) {
        console.error('❌ 网络错误:', error);
        return null;
    }
}

// ============================================
// 使用示例
// ============================================

async function main() {
    console.log('========== 闲鱼专属 API 测试 ==========\n');

    // 示例1: 提交单个账号
    console.log('示例1: 提交单个账号的邮箱信息');
    const singleAccount = {
        username: 'test@example.com',
        password: 'password123',
        outlook_email: 'test@outlook.com',
        mail_client_id: 'your-client-id-here',
        mail_refresh_token: 'your-refresh-token-here'
    };

    const result = await submitAccountEmailInfo(singleAccount);
    console.log('提交结果:', result);

    // 示例2: 批量提交多个账号
    console.log('\n\n示例2: 批量提交多个账号');
    const accounts = [
        {
            username: 'user1@example.com',
            password: 'pass1',
            outlook_email: 'user1@outlook.com',
            mail_client_id: 'client-id-1',
            mail_refresh_token: 'token-1'
        },
        {
            username: 'user2@example.com',
            password: 'pass2',
            outlook_email: 'user2@outlook.com',
            mail_client_id: 'client-id-2',
            mail_refresh_token: 'token-2'
        }
    ];

    await batchSubmitAccounts(accounts);

    // 示例3: 测试获取验证码（需要先提交过邮箱信息）
    console.log('\n\n示例3: 测试获取邮箱验证码');
    const accountId = await getAccountId('test@example.com');
    if (accountId) {
        const code = await getEmailVerificationCode(accountId);
        console.log('最终验证码:', code);
    }
}

// 在 Node.js 环境中运行
if (typeof require !== 'undefined' && require.main === module) {
    // 需要安装 node-fetch: npm install node-fetch
    global.fetch = require('node-fetch');
    main().catch(console.error);
}

// 导出供其他模块使用
if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        submitAccountEmailInfo,
        batchSubmitAccounts,
        getAccountId,
        getEmailVerificationCode
    };
}
