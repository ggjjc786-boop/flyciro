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
  }
};
