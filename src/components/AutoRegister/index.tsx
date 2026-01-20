import { useEffect, useState } from 'react';
import { Clock } from 'lucide-react';
import { AccountsTable } from './AccountsTable';
import { ImportPanel } from './ImportPanel';
import { ControlPanel } from './ControlPanel';
import { useStore } from '../../stores/autoRegisterStore';
import { api } from '../../api/autoRegister';
import { showError } from '../../utils/dialog';
import { useTheme } from '../../contexts/ThemeContext';

interface AutoRegisterProps {
  expireTime?: string;
}

export function AutoRegister({ expireTime }: AutoRegisterProps) {
  const { setAccounts, accounts, setSettings } = useStore();
  const [statusFilter, setStatusFilter] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const { colors, theme } = useTheme();
  const isDark = theme === 'dark';

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

  // 格式化到期时间
  const formatExpireTime = (timestamp?: string) => {
    if (!timestamp) return '';
    try {
      const date = new Date(parseInt(timestamp) * 1000);
      return date.toLocaleString('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return timestamp;
    }
  };

  return (
    <div className={`h-full overflow-hidden ${colors.main}`}>
      {/* 顶部到期时间显示 */}
      {expireTime && (
        <div className={`px-6 py-3 border-b ${isDark ? 'border-gray-700' : 'border-gray-200'}`}>
          <div className="flex items-center justify-end gap-2">
            <Clock className={`w-4 h-4 ${colors.textMuted}`} />
            <span className={`text-sm ${colors.textMuted}`}>
              到期时间: <span className={colors.text}>{formatExpireTime(expireTime)}</span>
            </span>
          </div>
        </div>
      )}
      
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
