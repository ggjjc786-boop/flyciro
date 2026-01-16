import { Moon, Sun, Minus, Square, X, LogOut, Clock } from 'lucide-react';
import { useStore } from '../store';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { checkLocalVerify, getRemainingTime, kamiUnbind } from '../kamiApi';
import { useState, useEffect } from 'react';
import './TitleBar.css';

export function TitleBar() {
  const { theme, setTheme, titleBarVisible } = useStore();
  const [remainingTime, setRemainingTime] = useState('');

  useEffect(() => {
    const updateRemainingTime = () => {
      const verifyInfo = checkLocalVerify();
      if (verifyInfo) {
        setRemainingTime(getRemainingTime(verifyInfo.vipExpireTime));
      }
    };

    updateRemainingTime();
    const interval = setInterval(updateRemainingTime, 60000); // 每分钟更新
    return () => clearInterval(interval);
  }, []);

  const toggleTheme = () => {
    setTheme(theme === 'light' ? 'dark' : 'light');
  };

  const handleLogout = async () => {
    if (confirm('确定要退出登录吗？')) {
      await kamiUnbind();
      window.location.reload();
    }
  };

  const handleMinimize = async () => {
    const appWindow = getCurrentWindow();
    await appWindow.minimize();
  };

  const handleMaximize = async () => {
    const appWindow = getCurrentWindow();
    const isMaximized = await appWindow.isMaximized();
    if (isMaximized) {
      await appWindow.unmaximize();
    } else {
      await appWindow.maximize();
    }
  };

  const handleClose = async () => {
    const appWindow = getCurrentWindow();
    await appWindow.close();
  };

  return (
    <div className={`title-bar ${titleBarVisible ? 'visible' : 'hidden'}`}>
      <div className="title-bar-content" data-tauri-drag-region>
        <div className="title-bar-left">
          <h1 className="title-bar-title">AWS Builder ID 自动注册系统</h1>
          {remainingTime && (
            <span className="vip-time">
              <Clock size={14} />
              <span>到期: {remainingTime}</span>
            </span>
          )}
        </div>
        <div className="title-bar-right">
          <button
            className="title-bar-button"
            onClick={handleLogout}
            title="退出登录"
          >
            <LogOut size={18} />
          </button>
          <button
            className="title-bar-button"
            onClick={toggleTheme}
            title={theme === 'light' ? '切换到暗黑模式' : '切换到亮色模式'}
          >
            {theme === 'light' ? <Moon size={18} /> : <Sun size={18} />}
          </button>
          <button
            className="title-bar-button window-button"
            onClick={handleMinimize}
            title="最小化"
          >
            <Minus size={18} />
          </button>
          <button
            className="title-bar-button window-button"
            onClick={handleMaximize}
            title="最大化/还原"
          >
            <Square size={16} />
          </button>
          <button
            className="title-bar-button window-button close-button"
            onClick={handleClose}
            title="关闭"
          >
            <X size={18} />
          </button>
        </div>
      </div>
    </div>
  );
}
