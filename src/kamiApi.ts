// 极简云卡密验证 API - 通过 Tauri 后端调用
import { invoke } from '@tauri-apps/api/core';

// 生成设备码
export function getDeviceCode(): string {
  let deviceCode = localStorage.getItem('device_code');
  if (!deviceCode) {
    deviceCode = 'DEV_' + Date.now().toString(36) + '_' + Math.random().toString(36).substring(2, 11);
    localStorage.setItem('device_code', deviceCode);
  }
  return deviceCode;
}

// 验证信息
export interface VerifyInfo {
  kami: string;
  vipExpireTime: number;
  verifyTime: number;
}

// 后端响应类型
interface KamiResponse {
  success: boolean;
  message: string;
  vip_expire_time: number | null;
}

// 卡密登录验证
export async function kamiLogin(kami: string): Promise<{ success: boolean; message: string; data?: VerifyInfo }> {
  try {
    const deviceCode = getDeviceCode();
    
    const result: KamiResponse = await invoke('kami_login', {
      kami: kami,
      markcode: deviceCode
    });
    
    if (result.success && result.vip_expire_time) {
      const verifyInfo: VerifyInfo = {
        kami: kami,
        vipExpireTime: result.vip_expire_time,
        verifyTime: Math.floor(Date.now() / 1000)
      };
      
      localStorage.setItem('kami_verify', JSON.stringify(verifyInfo));
      localStorage.setItem('kami_value', kami);
      
      return {
        success: true,
        message: '验证成功',
        data: verifyInfo
      };
    } else {
      return {
        success: false,
        message: result.message || '验证失败'
      };
    }
  } catch (error) {
    return {
      success: false,
      message: `验证错误: ${error}`
    };
  }
}

// 卡密解绑
export async function kamiUnbind(): Promise<{ success: boolean; message: string }> {
  try {
    const deviceCode = getDeviceCode();
    const savedKami = getSavedKami();
    
    const result: KamiResponse = await invoke('kami_unbind', {
      kami: savedKami,
      markcode: deviceCode
    });
    
    if (result.success) {
      localStorage.removeItem('kami_verify');
      localStorage.removeItem('kami_value');
    }
    
    return {
      success: result.success,
      message: result.message
    };
  } catch (error) {
    return {
      success: false,
      message: `解绑错误: ${error}`
    };
  }
}

// 获取公告
export async function getNotice(): Promise<{ success: boolean; content: string }> {
  try {
    const content: string = await invoke('get_notice');
    return {
      success: true,
      content: content || ''
    };
  } catch (error) {
    return {
      success: false,
      content: ''
    };
  }
}

// 检查本地验证状态
export function checkLocalVerify(): VerifyInfo | null {
  try {
    const verifyStr = localStorage.getItem('kami_verify');
    if (!verifyStr) return null;
    
    const verifyInfo: VerifyInfo = JSON.parse(verifyStr);
    const now = Math.floor(Date.now() / 1000);
    
    if (verifyInfo.vipExpireTime < now) {
      localStorage.removeItem('kami_verify');
      localStorage.removeItem('kami_value');
      return null;
    }
    
    return verifyInfo;
  } catch {
    return null;
  }
}

// 获取保存的卡密
export function getSavedKami(): string {
  return localStorage.getItem('kami_value') || '';
}

// 格式化到期时间
export function formatExpireTime(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  });
}

// 计算剩余时间
export function getRemainingTime(expireTimestamp: number): string {
  const now = Math.floor(Date.now() / 1000);
  const remaining = expireTimestamp - now;
  
  if (remaining <= 0) return '已过期';
  
  const days = Math.floor(remaining / 86400);
  const hours = Math.floor((remaining % 86400) / 3600);
  const minutes = Math.floor((remaining % 3600) / 60);
  
  if (days > 0) {
    return `${days}天${hours}小时`;
  } else if (hours > 0) {
    return `${hours}小时${minutes}分钟`;
  } else {
    return `${minutes}分钟`;
  }
}
