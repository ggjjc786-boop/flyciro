import { useEffect, useState } from 'react';
import { AccountsTable } from './AccountsTable';
import { ImportPanel } from './ImportPanel';
import { ControlPanel } from './ControlPanel';
import { useStore } from '../../stores/autoRegisterStore';
import { api } from '../../api/autoRegister';
import { showError } from '../../utils/dialog';
import { useTheme } from '../../contexts/ThemeContext';

export function AutoRegister() {
  const { setAccounts, accounts, setSettings } = useStore();
  const [statusFilter, setStatusFilter] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const { colors } = useTheme();

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
    <div className={`h-full overflow-hidden ${colors.main}`}>
      <div className="flex gap-6 h-full p-6">
        <div className="w-96 flex-shrink-0 flex flex-col gap-6 overflow-y-auto">
          <ImportPanel onImportComplete={loadData} />
          <ControlPanel
            onFilterChange={handleFilterChange}
            onRefresh={loadData}
          />
        </div>

        <div className="flex-1 min-w-0 flex flex-col">
          {isLoading ? (
            <div className="flex flex-col items-center justify-center h-full gap-4">
              <div className="spinner"></div>
              <p className={colors.textMuted}>加载中...</p>
            </div>
          ) : (
            <AccountsTable accounts={accounts} onRefresh={loadData} />
          )}
        </div>
      </div>
    </div>
  );
}
