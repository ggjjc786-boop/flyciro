/**
 * Cursor Service API 客户端
 * 用于与 Python 后端服务通信
 */

const CURSOR_API_BASE = 'http://127.0.0.1:8765/api/cursor';

class CursorApiClient {
  constructor(baseUrl = CURSOR_API_BASE) {
    this.baseUrl = baseUrl;
  }

  async request(endpoint, options = {}) {
    const url = `${this.baseUrl}${endpoint}`;
    const defaultOptions = {
      headers: {
        'Content-Type': 'application/json',
      },
    };

    try {
      const response = await fetch(url, { ...defaultOptions, ...options });
      const data = await response.json();
      
      if (!response.ok) {
        throw new Error(data.detail || data.message || '请求失败');
      }
      
      return data;
    } catch (error) {
      if (error.message === 'Failed to fetch') {
        throw new Error('无法连接到 Cursor 服务，请确保服务已启动');
      }
      throw error;
    }
  }

  // ==================== 账号管理 ====================

  async getAccounts(page = 1, perPage = 20, search = '') {
    const params = new URLSearchParams({ page, per_page: perPage });
    if (search) params.append('search', search);
    return this.request(`/accounts?${params}`);
  }

  async createAccount(account) {
    return this.request('/accounts', {
      method: 'POST',
      body: JSON.stringify(account),
    });
  }

  async deleteAccount(accountId) {
    return this.request(`/accounts/${accountId}`, {
      method: 'DELETE',
    });
  }

  async checkAccountStatus(accountId) {
    return this.request(`/accounts/${accountId}/status`);
  }

  // ==================== 机器码 ====================

  async getMachineId() {
    return this.request('/machine-id');
  }

  async resetMachineId() {
    return this.request('/machine-id/reset', {
      method: 'POST',
    });
  }

  // ==================== 注册 ====================

  async getRegistrationStatus() {
    return this.request('/registration/status');
  }

  async getRegistrationLogs() {
    return this.request('/registration/logs');
  }

  // ==================== 验证码 ====================

  async getPendingVerifications() {
    return this.request('/verification/pending');
  }

  async submitVerificationCode(emailId, code) {
    return this.request('/verification/submit', {
      method: 'POST',
      body: JSON.stringify({ email_id: emailId, code }),
    });
  }

  // ==================== 配置 ====================

  async getConfig() {
    return this.request('/config');
  }

  async updateConfig(config) {
    return this.request('/config', {
      method: 'POST',
      body: JSON.stringify(config),
    });
  }

  // ==================== 健康检查 ====================

  async healthCheck() {
    return this.request('/health');
  }
}

export const cursorApi = new CursorApiClient();
export default cursorApi;
