import { useState, useEffect } from 'react';
import { kamiLogin, getNotice, getSavedKami, getDeviceCode } from '../kamiApi';
import './LoginPanel.css';

interface LoginPanelProps {
  onLoginSuccess: () => void;
}

export function LoginPanel({ onLoginSuccess }: LoginPanelProps) {
  const [kami, setKami] = useState('');
  const [notice, setNotice] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const [deviceCode, setDeviceCode] = useState('');

  useEffect(() => {
    // 加载保存的卡密
    const savedKami = getSavedKami();
    if (savedKami) {
      setKami(savedKami);
    }
    
    // 获取设备码
    setDeviceCode(getDeviceCode());
    
    // 获取公告
    loadNotice();
  }, []);

  const loadNotice = async () => {
    const result = await getNotice();
    if (result.success) {
      setNotice(result.content);
    }
  };

  const handleLogin = async () => {
    if (!kami.trim()) {
      setError('请输入卡密');
      return;
    }

    setIsLoading(true);
    setError('');

    const result = await kamiLogin(kami.trim());

    setIsLoading(false);

    if (result.success) {
      onLoginSuccess();
    } else {
      setError(result.message);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleLogin();
    }
  };

  return (
    <div className="login-panel">
      <div className="login-container">
        <div className="login-header">
          <h1>AWS Builder 自动化工具</h1>
          <p className="login-subtitle">请输入卡密验证后使用</p>
        </div>

        {notice && (
          <div className="notice-box">
            <div className="notice-title">📢 公告</div>
            <div className="notice-content">{notice}</div>
          </div>
        )}

        <div className="login-form">
          <div className="form-group">
            <label>卡密</label>
            <input
              type="text"
              value={kami}
              onChange={(e) => setKami(e.target.value)}
              onKeyPress={handleKeyPress}
              placeholder="请输入卡密"
              disabled={isLoading}
            />
          </div>

          {error && <div className="error-message">{error}</div>}

          <button
            className="login-button"
            onClick={handleLogin}
            disabled={isLoading}
          >
            {isLoading ? '验证中...' : '登录验证'}
          </button>
        </div>

        <div className="device-info">
          <span className="device-label">设备码:</span>
          <span className="device-code">{deviceCode}</span>
        </div>

        <div className="login-footer">
          <p>购买卡密请联系管理员</p>
        </div>
      </div>
    </div>
  );
}
