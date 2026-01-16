// 极简云卡密验证 API
// 接口地址: https://zh.xphdfs.me/api.php

const API_BASE = 'https://zh.xphdfs.me/api.php';
const APP_ID = '10002';  // 应用APPID
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const APP_KEY = 'DxhTVxT08L0AD3Dx';  // 应用密钥（预留）
const RC4_KEY = '8HacPHMcsWK10002';  // RC4密钥

// RC4加密/解密
function rc4(key: string, data: string): string {
  const s: number[] = [];
  let j = 0;
  let result = '';

  for (let i = 0; i < 256; i++) {
    s[i] = i;
  }

  for (let i = 0; i < 256; i++) {
    j = (j + s[i] + key.charCodeAt(i % key.length)) % 256;
    [s[i], s[j]] = [s[j], s[i]];
  }

  let i = 0;
  j = 0;
  for (let k = 0; k < data.length; k++) {
    i = (i + 1) % 256;
    j = (j + s[i]) % 256;
    [s[i], s[j]] = [s[j], s[i]];
    const t = (s[i] + s[j]) % 256;
    result += String.fromCharCode(data.charCodeAt(k) ^ s[t]);
  }

  return result;
}

// RC4加密并转为十六进制（导出供外部使用）
export function rc4Encrypt(data: string): string {
  const encrypted = rc4(RC4_KEY, data);
  let hex = '';
  for (let i = 0; i < encrypted.length; i++) {
    hex += encrypted.charCodeAt(i).toString(16).padStart(2, '0');
  }
  return hex;
}

// 从十六进制解密（导出供外部使用）
export function rc4Decrypt(hexData: string): string {
  let data = '';
  for (let i = 0; i < hexData.length; i += 2) {
    data += String.fromCharCode(parseInt(hexData.substr(i, 2), 16));
  }
  return rc4(RC4_KEY, data);
}

// 生成设备码
export function getDeviceCode(): string {
  let deviceCode = localStorage.getItem('device_code');
  if (!deviceCode) {
    // 生成唯一设备码
    deviceCode = 'DEV_' + Date.now().toString(36) + '_' + Math.random().toString(36).substr(2, 9);
    localStorage.setItem('device_code', deviceCode);
  }
  return deviceCode;
}

// 卡密验证响应
export interface KamiLoginResponse {
  code: number;
  msg: {
    kami: string;
    vip: string;  // 到期时间戳
  } | string;
  time: number;
  check: string;
}

// 解绑响应
export interface KamiUnbindResponse {
  code: number;
  msg: string;
  time: number;
  check: string;
}

// 公告响应
export interface NoticeResponse {
  code: number;
  msg: {
    app_gg: string;
  };
  time: number;
  check: string;
}

// 验证信息
export interface VerifyInfo {
  kami: string;
  vipExpireTime: number;  // 到期时间戳
  verifyTime: number;     // 验证时间
}

// 卡密登录验证
export async function kamiLogin(kami: string): Promise<{ success: boolean; message: string; data?: VerifyInfo }> {
  try {
    const deviceCode = getDeviceCode();
    const timestamp = Math.floor(Date.now() / 1000);
    
    const url = `${API_BASE}?api=kmlogon&app=${APP_ID}&kami=${encodeURIComponent(kami)}&markcode=${encodeURIComponent(deviceCode)}&t=${timestamp}`;
    
    const response = await fetch(url);
    const result: KamiLoginResponse = await response.json();
    
    if (result.code === 200 && typeof result.msg === 'object') {
      const verifyInfo: VerifyInfo = {
        kami: result.msg.kami,
        vipExpireTime: parseInt(result.msg.vip),
        verifyTime: result.time
      };
      
      // 保存验证信息到本地
      localStorage.setItem('kami_verify', JSON.stringify(verifyInfo));
      localStorage.setItem('kami_value', kami);
      
      return {
        success: true,
        message: '验证成功',
        data: verifyInfo
      };
    } else {
      const errorMsg = typeof result.msg === 'string' ? result.msg : '验证失败';
      return {
        success: false,
        message: errorMsg
      };
    }
  } catch (error) {
    return {
      success: false,
      message: `网络错误: ${error}`
    };
  }
}

// 卡密解绑
export async function kamiUnbind(): Promise<{ success: boolean; message: string }> {
  try {
    const deviceCode = getDeviceCode();
    const timestamp = Math.floor(Date.now() / 1000);
    
    const url = `${API_BASE}?api=kmunmachine&app=${APP_ID}&markcode=${encodeURIComponent(deviceCode)}&t=${timestamp}`;
    
    const response = await fetch(url);
    const result: KamiUnbindResponse = await response.json();
    
    if (result.code === 200) {
      // 清除本地验证信息
      localStorage.removeItem('kami_verify');
      localStorage.removeItem('kami_value');
      
      return {
        success: true,
        message: result.msg || '解绑成功'
      };
    } else {
      return {
        success: false,
        message: result.msg || '解绑失败'
      };
    }
  } catch (error) {
    return {
      success: false,
      message: `网络错误: ${error}`
    };
  }
}

// 获取公告
export async function getNotice(): Promise<{ success: boolean; content: string }> {
  try {
    const url = `${API_BASE}?api=notice&app=${APP_ID}`;
    
    const response = await fetch(url);
    const result: NoticeResponse = await response.json();
    
    if (result.code === 200) {
      return {
        success: true,
        content: result.msg.app_gg
      };
    } else {
      return {
        success: false,
        content: '获取公告失败'
      };
    }
  } catch (error) {
    return {
      success: false,
      content: `网络错误: ${error}`
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
    
    // 检查是否过期
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
