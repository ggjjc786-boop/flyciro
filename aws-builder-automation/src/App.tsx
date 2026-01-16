import { useEffect, useState, useRef } from 'react';
import { TitleBar } from './components/TitleBar';
import { AccountsTable } from './components/AccountsTable';
import { ImportPanel } from './components/ImportPanel';
import { ControlPanel } from './components/ControlPanel';
import { useStore } from './store';
import { api } from './api';
import { showError } from './utils/dialog';
import './App.css';
import './index.css';

function App() {
  const { theme, setAccounts, accounts, setSettings, setTitleBarVisible } = useStore();
  const [statusFilter, setStatusFilter] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const contentRef = useRef<HTMLDivElement>(null);
  const lastScrollY = useRef(0);

  useEffect(() => {
    // Set initial theme
    document.documentElement.setAttribute('data-theme', theme);

    // Load initial data
    loadData();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    // Handle scroll for auto-hide title bar
    const handleScroll = () => {
      if (!contentRef.current) return;

      const currentScrollY = contentRef.current.scrollTop;

      if (currentScrollY > lastScrollY.current && currentScrollY > 60) {
        // Scrolling down
        setTitleBarVisible(false);
      } else if (currentScrollY < lastScrollY.current) {
        // Scrolling up
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

  return (
    <div className="app">
      <TitleBar />

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
    </div>
  );
}

export default App;
