import { invoke } from '@tauri-apps/api/core';

export const cardAuthApi = {
  /**
   * 获取公告
   */
  async getNotice(): Promise<string> {
    return invoke('get_card_notice');
  },

  /**
   * 验证卡密
   */
  async verifyCardKey(cardKey: string): Promise<string> {
    return invoke('verify_card_key', { cardKey });
  },

  /**
   * 解绑卡密
   */
  async unbindCardKey(cardKey: string): Promise<string> {
    return invoke('unbind_card_key', { cardKey });
  },

  /**
   * 获取设备码
   */
  async getDeviceCode(): Promise<string> {
    return invoke('get_device_code');
  },
};
