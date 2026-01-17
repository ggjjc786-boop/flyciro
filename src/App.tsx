import { useEffect, useState, useRef } from 'react';
import { Window } from '@tauri-apps/api/window';
import { Minus, X, Sparkles, MousePointer2, Layers } from 'lucide-react';
import { useStore } from './store';
import { api } from './api';
import { showError } from './utils/dialog';
import { checkLocalVerify, kamiLogin, getSavedKami } from './kamiApi';
import { useMode } from './contexts/ModeContext';

// AWS 模块组件
import { TitleBar } from './components/TitleBar';
import { AccountsTable } from './components/AccountsTable';
import { ImportPanel } from './components/ImportPanel';
import { ControlPanel } from './components/ControlPanel';
import { LoginPanel } from './components/LoginPanel';

// Kiro 模块组件 (从 kiro 文件夹导入)
import KiroSidebar from './components/kiro/Sidebar';
import KiroHome from './components/kiro/Home';
import KiroAccountManager from './components/kiro/AccountManager/index';
import KiroSettings from './components/kiro/Settings';
import KiroConfig from './components/kiro/KiroConfig/index';
import KiroAbout from './components/kiro/About';
import KiroLogin from './components/kiro/Login';
import KiroWebOAuthLogin from './components/kiro/WebOAuthLogin';
import KiroAuthCallback from './components/kiro/AuthCallback';
import KiroUserAuth from './components/kiro/UserAuth';
import KiroRegister from './components/kiro/Register';
import KiroUpdateChecker from './components/kiro/UpdateChecker';

import './App.css';

function App() {
  const { theme, setAccounts, accounts, setSettings, setTitleBarVisible } = useStore();
  const { mode, toggleMode } = useMode();
  
  // AWS 模块状态
  const [statusFilter, setStatusFilter] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isVerified, setIsVerified] = useState(false);
  const [isCheckingVerify, setIsCheckingVerify] = useState(true);
  
  // Kiro 模块状态
  const [kiroUser, setKiroUser] = useState(null);
  const [kiroLoading, setKiroLoading] = useState(true);
  const [kiroActiveMenu, setKiroActiveMenu] = useState('home');
  const [kiroUserInfo, setKiroUserInfo] = useState<any>(null);
  
  const contentRef = useRef<HTMLDivElement>(null);
  const lastScrollY = useRef(0);
  const refreshTimerRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
    
    if (mode === 'aws') {
      checkVerifyStatus();
    } else {
      initKiroMode();
    }
  }, [theme, mode]);

  // AWS 模式初始化
  const checkVerifyStatus = async () => {
    setIsCheckingVerify(true);
    
    const localVerify = checkLocalVerify();
    if (localVerify) {
      const savedKami = getSavedKami();
      if (savedKami) {
        const result = await kamiLogin(savedKami);
        if (result.success) {
          setIsVerified(true);
          setIsCheckingVerify(false);
          loadData();
          return;
        }
      }
    }
    
    setIsVerified(false);
    setIsCheckingVerify(false);
  };

  // Kiro 模式初始化
  const initKiroMode = async () => {
    const savedUser = localStorage.getItem('kiro_user');
    if (savedUser) {
      try {
        setKiroUserInfo(JSON.parse(savedUser));
      } catch (e) {
        localStorage.removeItem('kiro_user');
      }
    }
    
    setKiroLoading(false);
    
    const url = new URL(window.location.href);
    if (url.pathname === '/callback' && (url.searchParams.has('code') || url.searchParams.has('state'))) {
      setKiroActiveMenu('callback');
    }
  };

  const handleLoginSuccess = () => {
    setIsVerified(true);
    loadData();
  };

  const handleKiroUserVerified = (data: any) => {
    setKiroUserInfo({
      email: data.code?.includes('@') ? data.code : null,
      phone: data.code?.includes('@') ? null : data.code,
      nickname: data.nickname,
      is_vip: data.is_vip || false,
      vip_expires_at: data.expires_at
    });
    setKiroActiveMenu('home');
  };

  const handleKiroUserLogout = () => {
    localStorage.removeItem('kiro_user');
    localStorage.removeItem('kiro_token');
    setKiroUserInfo(null);
  };

  useEffect(() => {
    const handleScroll = () => {
      if (!contentRef.current) return;

      const currentScrollY = contentRef.current.scrollTop;

      if (currentScrollY > lastScrollY.current && currentScrollY > 60) {
        setTitleBarVisible(false);
      } else if (currentScrollY < lastScrollY.current) {
        setTitleBarVisible(true);
      }

      lastScrollY.current = currentScrollY;
    };

    const contentElement = contentRef.current;
    if (contentElement) {
      contentElement.addEventListener('scroll', handleScroll);
      return () => contentElement.removeEventListener('scroll', handleScroll);
    }
  }, [setTitleBarVisible]);

  const loadData = async () => {
    setIsLoading(true);
    try {
      const [accountsData, settingsData] = await Promise.all([
        api.getAccounts(statusFilter || undefined),
        api.getSettings(),
      ]);

      setAccounts(accountsData);
      setSettings(settingsData);
    } catch (error) {
      console.error('Failed to load data:', error);
      await showError('加载数据失败: ' + error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleFilterChange = async (filter: string | null) => {
    setStatusFilter(filter);
    setIsLoading(true);
    try {
      const accountsData = await api.getAccounts(filter || undefined);
      setAccounts(accountsData);
    } catch (error) {
      console.error('Failed to load filtered data:', error);
      await showError('加载数据失败: ' + error);
    } finally {
      setIsLoading(false);
    }
  };

  const renderKiroContent = () => {
    switch (kiroActiveMenu) {
      case 'home': return <KiroHome onNavigate={setKiroActiveMenu} />;
      case 'token': return <KiroAccountManager />;
      case 'register': return <KiroRegister userInfo={kiroUserInfo} />;
      case 'kiro-config': return <KiroConfig />;
      case 'login': return <KiroLogin onLogin={(u: any) => { setKiroUser(u); setKiroActiveMenu('token'); }} />;
      case 'web-oauth': return <KiroWebOAuthLogin onLogin={(u: any) => { setKiroUser(u); setKiroActiveMenu('token'); }} />;
      case 'callback': return <KiroAuthCallback />;
      case 'settings': return <KiroSettings />;
      case 'about': return <KiroAbout />;
      case 'user-auth': return <KiroUserAuth onVerified={handleKiroUserVerified} onBack={() => setKiroActiveMenu('home')} />;
      default: return <KiroHome onNavigate={setKiroActiveMenu} />;
    }
  };

  if (mode === 'aws' && isCheckingVerify) {
    return (
      <div className="h-screen bg-[#0c0c14] flex items-center justify-center relative">
        <div className="aurora-bg">
          <div className="aurora-blob aurora-blob-1" />
          <div className="aurora-blob aurora-blob-2" />
        </div>
        <div className="text-center animate-fade-in relative z-10">
          <div className="w-20 h-20 bg-gradient-to-br from-cyan-500 to-blue-600 rounded-3xl flex items-center justify-center mx-auto mb-6 shadow-2xl shadow-cyan-500/30">
            <MousePointer2 size={36} className="text-white" />
          </div>
          <div className="text-white/80 font-medium text-lg">正在验证...</div>
        </div>
      </div>
    );
  }

  if (mode === 'aws' && !isVerified) {
    return <LoginPanel onLoginSuccess={handleLoginSuccess} />;
  }

  if (mode === 'kiro' && kiroLoading) {
    return (
      <div className="h-screen bg-[#0c0c14] flex items-center justify-center relative">
        <div className="aurora-bg">
          <div className="aurora-blob aurora-blob-1" />
          <div className="aurora-blob aurora-blob-2" />
        </div>
        <div className="text-center animate-fade-in relative z-10">
          <div className="w-20 h-20 bg-gradient-to-br from-indigo-500 to-purple-600 rounded-3xl flex items-center justify-center mx-auto mb-6 shadow-2xl shadow-indigo-500/30">
            <Sparkles size={36} className="text-white" />
          </div>
          <div className="text-white/80 font-medium text-lg">加载中...</div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-screen bg-[#0c0c14] relative">
      {/* Aurora 背景 */}
      <div className="aurora-bg">
        <div className="aurora-blob aurora-blob-1" />
        <div className="aurora-blob aurora-blob-2" />
        <div className="aurora-blob aurora-blob-3" />
      </div>
      
      {/* 标题栏 */}
      <div 
        data-tauri-drag-region 
        className="h-10 flex items-center justify-between px-4 select-none glass-panel border-t-0 border-x-0 relative z-50"
      >
        <div className="flex items-center gap-3">
          <div className={`w-6 h-6 rounded-lg flex items-center justify-center ${
            mode === 'kiro' 
              ? 'bg-gradient-to-br from-indigo-500 to-purple-600' 
              : 'bg-gradient-to-br from-cyan-500 to-blue-600'
          }`}>
            {mode === 'kiro' ? (
              <Sparkles size={14} className="text-white" />
            ) : (
              <MousePointer2 size={14} className="text-white" />
            )}
          </div>
          <span className="text-sm font-semibold text-white/90">
            {mode === 'kiro' ? 'Kiro Account Manager' : 'AWS Builder ID Tool'}
          </span>
          <span className="text-xs text-white/30">v2.0</span>
        </div>
        <div className="flex items-center gap-3">
          {/* 模式切换 */}
          <button
            onClick={toggleMode}
            className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 transition-colors"
            title="切换模式"
          >
            <Layers size={14} className="text-white/60" />
            <span className="text-xs text-white/60">{mode === 'kiro' ? 'Kiro' : 'AWS'}</span>
          </button>
          
          {/* 窗口控制 */}
          <div className="flex items-center gap-1">
            <button
              onClick={async () => await Window.getCurrent().minimize()}
              className="w-9 h-9 flex items-center justify-center rounded-lg hover:bg-white/10 transition-colors"
            >
              <Minus size={16} className="text-white/60" />
            </button>
            <button
              onClick={async () => await Window.getCurrent().close()}
              className="w-9 h-9 flex items-center justify-center rounded-lg hover:bg-red-500/80 transition-colors group"
            >
              <X size={16} className="text-white/60 group-hover:text-white" />
            </button>
          </div>
        </div>
      </div>
      
      {/* 主内容 */}
      <div className="flex flex-1 overflow-hidden relative z-10">
        {mode === 'kiro' ? (
          <>
            <KiroSidebar 
              activeMenu={kiroActiveMenu} 
              onMenuChange={setKiroActiveMenu}
              userInfo={kiroUserInfo}
              onUserLogout={handleKiroUserLogout}
            />
            <main className="flex-1 overflow-hidden">
              {renderKiroContent()}
            </main>
            <KiroUpdateChecker />
          </>
        ) : (
          <>
            <div className="app-content" ref={contentRef}>
              <div className="app-main">
                <div className="app-sidebar">
                  <ImportPanel onImportComplete={loadData} />
                  <ControlPanel
                    onFilterChange={handleFilterChange}
                    onRefresh={loadData}
                  />
                </div>

                <div className="app-table-section">
                  {isLoading ? (
                    <div className="loading-container">
                      <div className="spinner"></div>
                      <p>加载中...</p>
                    </div>
                  ) : (
                    <AccountsTable accounts={accounts} onRefresh={loadData} />
                  )}
                </div>
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}

export default App;
