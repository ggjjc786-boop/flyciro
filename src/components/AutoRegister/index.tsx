import { useEffect, useState, useRef } from 'react';
import { AccountsTable } from './AccountsTable';
import { ImportPanel } from './ImportPanel';
import { ControlPanel } from './ControlPanel';
import { useStore } from '../../stores/autoRegisterStore';
import { api } from '../../api/autoRegister';
import { showError } from '../../utils/dialog';
import './AutoRegister.css';

export function AutoRegister() {
  const { setAccounts, accounts, setSettings } = useStore();
  const [statusFilter, setStatusFilter] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    loadData();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

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
    <div className="auto-register-container">
      <div className="auto-register-sidebar">
        <ImportPanel onImportComplete={loadData} />
        <ControlPanel
          onFilterChange={handleFilterChange}
          onRefresh={loadData}
        />
      </div>

      <div className="auto-register-main">
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
  );
}
