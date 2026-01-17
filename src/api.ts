import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { readTextFile, writeTextFile } from '@tauri-apps/plugin-fs';
import { Account, Settings } from './store';

export interface NewAccount {
  email: string;
  email_password: string;
  client_id: string;
  refresh_token: string;
}

export interface AccountUpdate {
  id: number;
  email?: string;
  email_password?: string;
  client_id?: string;
  refresh_token?: string;
  kiro_password?: string;
  status?: 'not_registered' | 'in_progress' | 'registered' | 'error';
  error_reason?: string;
}

export interface ImportResult {
  success_count: number;
  error_count: number;
  errors: Array<{
    line_number: number;
    content: string;
    reason: string;
  }>;
}

export interface EmailMessage {
  id: string;
  received_datetime: string;
  sent_datetime: string;
  subject: string;
  body_content: string;
  from_address: string;
}

export const api = {
  async getAccounts(statusFilter?: string): Promise<Account[]> {
    return invoke('get_accounts', { statusFilter });
  },

  async addAccount(account: NewAccount): Promise<number> {
    return invoke('add_account', { account });
  },

  async updateAccount(update: AccountUpdate): Promise<void> {
    return invoke('update_account', { update });
  },

  async deleteAccount(id: number): Promise<void> {
    return invoke('delete_account', { id });
  },

  async deleteAllAccounts(): Promise<void> {
    return invoke('delete_all_accounts');
  },

  async importAccounts(content: string): Promise<ImportResult> {
    return invoke('import_accounts', { content });
  },

  async getSettings(): Promise<Settings> {
    return invoke('get_settings');
  },

  async updateSettings(settings: Settings): Promise<void> {
    return invoke('update_settings', { settings });
  },

  async startRegistration(accountId: number): Promise<string> {
    return invoke('start_registration', { accountId });
  },

  async startBatchRegistration(): Promise<string> {
    return invoke('start_batch_registration');
  },

  async exportAccounts(statusFilter?: string): Promise<void> {
    const content: string = await invoke('export_accounts', { statusFilter });

    if (!content) {
      throw new Error('没有可导出的数据');
    }

    const filePath = await save({
      filters: [{
        name: 'Text Files',
        extensions: ['txt']
      }],
      defaultPath: 'accounts.txt'
    });

    if (filePath) {
      await writeTextFile(filePath, content);
    }
  },

  async selectFile(): Promise<string | null> {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Text Files',
        extensions: ['txt']
      }]
    });

    if (selected && typeof selected === 'string') {
      const content = await readTextFile(selected);
      return content;
    }

    return null;
  },

  async fetchEmails(accountId: number, _maxResults?: number): Promise<EmailMessage[]> {
    return invoke('fetch_latest_email', { accountId });
  },

  // 同步账号到服务器
  async syncToServer(serverUrl: string, secretKey: string): Promise<{ success: boolean; message: string; successCount?: number; failCount?: number }> {
    try {
      // 获取所有账号
      const accounts: Account[] = await invoke('get_accounts', {});
      
      if (accounts.length === 0) {
        return { success: false, message: '没有账号可同步' };
      }
      
      // 准备同步数据
      const syncData = accounts.map(acc => ({
        email: acc.email,
        email_password: acc.email_password,
        client_id: acc.client_id,
        refresh_token: acc.refresh_token,
        kiro_password: acc.kiro_password || '',
        status: acc.status === 'in_progress' ? 'not_registered' : acc.status
      }));
      
      // 发送到服务器
      const response = await fetch(`${serverUrl}/api/sync/accounts`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          secret_key: secretKey,
          accounts: syncData
        })
      });
      
      const result = await response.json();
      
      if (result.success) {
        return {
          success: true,
          message: `同步成功! 成功: ${result.successCount}, 失败: ${result.failCount}`,
          successCount: result.successCount,
          failCount: result.failCount
        };
      } else {
        return { success: false, message: result.message || '同步失败' };
      }
    } catch (error) {
      return { success: false, message: `同步错误: ${error}` };
    }
  }
};

  // 批量获取 Kiro Token
  async batchFetchKiroTokens(): Promise<string> {
    return invoke('batch_fetch_kiro_tokens');
  },

  // 为单个账号获取 Kiro Token
  async autoFetchKiroToken(accountId: number): Promise<string> {
    return invoke('auto_fetch_kiro_token', { accountId });
  },
