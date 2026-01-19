import { useState } from 'react';
import { Filter, Settings, Loader2, Download, Cloud, Upload } from 'lucide-react';
import { api } from '../../api/autoRegister';
import { useStore } from '../../stores/autoRegisterStore';
import { showConfirm, showSuccess, showError } from '../../utils/dialog';
import { useTheme } from '../../contexts/ThemeContext';

interface ControlPanelProps {
  onFilterChange: (filter: string | null) => void;
  onRefresh: () => void;
}

const SYNC_CONFIG_KEY = 'xianyu_sync_config';

interface SyncConfig {
  serverUrl: string;
  secretKey: string;
}

function getSyncConfig(): SyncConfig {
  const saved = localStorage.getItem(SYNC_CONFIG_KEY);
  if (saved) {
    try {
      return JSON.parse(saved);
    } catch {
      // ignore
    }
  }
  return { serverUrl: '', secretKey: 'xianyu_sync_2024' };
}

function saveSyncConfig(config: SyncConfig) {
  localStorage.setItem(SYNC_CONFIG_KEY, JSON.stringify(config));
}

export function ControlPanel({ onFilterChange, onRefresh }: ControlPanelProps) {
  const { colors, theme } = useTheme();
  const isDark = theme === 'dark';
  const [selectedFilter, setSelectedFilter] = useState<string | null>(null);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isSyncModalOpen, setIsSyncModalOpen] = useState(false);
  const { settings, setSettings } = useStore();
  const [isLoading, setIsLoading] = useState(false);
  const [syncConfig, setSyncConfig] = useState<SyncConfig>(getSyncConfig());
  const [isSyncing, setIsSyncing] = useState(false);
  const [isImporting, setIsImporting] = useState(false);

  const filters = [
    { value: null, label: '全部', count: 0 },
    { value: 'not_registered', label: '未注册', count: 0 },
    { value: 'in_progress', label: '进行中', count: 0 },
    { value: 'registered', label: '已注册', count: 0 },
    { value: 'error', label: '异常', count: 0 },
  ];

  const handleFilterChange = (filter: string | null) => {
    setSelectedFilter(filter);
    onFilterChange(filter);
  };

  const handleDeleteAll = async () => {
    console.log('[ControlPanel] Delete all button clicked');
    try {
      const confirmed = await showConfirm(
        '确定要删除所有账号数据吗（包括进行中的账户）？此操作无法撤销！',
        '确认删除'
      );
      console.log('[ControlPanel] First confirmation:', confirmed);

      if (confirmed) {
        const doubleConfirmed = await showConfirm(
          '再次确认：这将永久删除所有账号数据（包括进行中状态）！',
          '最终确认'
        );
        console.log('[ControlPanel] Second confirmation:', doubleConfirmed);

        if (doubleConfirmed) {
          setIsLoading(true);
          try {
            console.log('[ControlPanel] Calling deleteAllAccounts API...');
            await api.deleteAllAccounts();
            console.log('[ControlPanel] Delete successful');
            await showSuccess('已成功删除所有数据');
            onRefresh();
          } catch (error) {
            console.error('[ControlPanel] Delete failed:', error);
            await showError('删除失败: ' + error);
          } finally {
            setIsLoading(false);
          }
        }
      }
    } catch (error) {
      console.error('[ControlPanel] Error in handleDeleteAll:', error);
    }
  };

  const handleSaveSettings = async () => {
    setIsLoading(true);
    try {
      await api.updateSettings(settings);
      await showSuccess('设置已保存');
      setIsSettingsOpen(false);
    } catch (error) {
      await showError('保存失败: ' + error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleBatchRegistration = async () => {
    console.log('[ControlPanel] Batch registration button clicked');
    try {
      const confirmed = await showConfirm(
        '确定要对所有未注册的账号进行批量注册吗？这将依次处理所有账号。',
        '批量注册确认'
      );
      console.log('[ControlPanel] Confirmation:', confirmed);

      if (confirmed) {
        setIsLoading(true);
        try {
          console.log('[ControlPanel] Calling startBatchRegistration API...');
          const result = await api.startBatchRegistration();
          console.log('[ControlPanel] Batch registration result:', result);
          await showSuccess(result);
          onRefresh();
        } catch (error) {
          console.error('[ControlPanel] Batch registration failed:', error);
          await showError('批量注册失败: ' + error);
        } finally {
          setIsLoading(false);
        }
      }
    } catch (error) {
      console.error('[ControlPanel] Error in handleBatchRegistration:', error);
    }
  };

  const handleExport = async () => {
    setIsLoading(true);
    try {
      await api.exportAccounts(selectedFilter || undefined);
      await showSuccess('导出成功');
    } catch (error) {
      await showError('导出失败: ' + error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleOpenSyncModal = () => {
    setSyncConfig(getSyncConfig());
    setIsSyncModalOpen(true);
  };

  const handleSync = async () => {
    if (!syncConfig.serverUrl) {
      await showError('请输入服务器地址');
      return;
    }
    if (!syncConfig.secretKey) {
      await showError('请输入同步密钥');
      return;
    }

    saveSyncConfig(syncConfig);

    setIsSyncing(true);
    try {
      const result = await api.syncToServer(syncConfig.serverUrl, syncConfig.secretKey);
      if (result.success) {
        await showSuccess(`同步成功！\n成功: ${result.successCount || 0} 个\n失败: ${result.failCount || 0} 个`);
        setIsSyncModalOpen(false);
      } else {
        await showError('同步失败: ' + result.message);
      }
    } catch (error) {
      await showError('同步失败: ' + error);
    } finally {
      setIsSyncing(false);
    }
  };

  const handleImportToMain = async () => {
    const confirmed = await showConfirm(
      '确定要将已注册的账号导入到主账号列表吗？',
      '导入确认'
    );

    if (confirmed) {
      setIsImporting(true);
      try {
        const result = await api.importToMain();
        await showSuccess(result);
        onRefresh();
      } catch (error) {
        await showError('导入失败: ' + error);
      } finally {
        setIsImporting(false);
      }
    }
  };

  return (
    <div className={`card-glow ${colors.card} rounded-2xl border ${colors.cardBorder} p-4 shadow-sm flex-shrink-0`}>
      <div className="mb-4">
        <div className="flex items-center gap-2 mb-3">
          <Filter size={16} className={colors.textMuted} />
          <span className={`text-xs font-semibold ${colors.textMuted}`}>状态筛选</span>
        </div>
        <div className="flex flex-wrap gap-2">
          {filters.map(filter => (
            <button
              key={filter.value || 'all'}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-all ${
                selectedFilter === filter.value
                  ? 'bg-blue-500 text-white shadow-sm'
                  : `${colors.card} ${colors.text} border ${colors.cardBorder} hover:scale-[1.02]`
              }`}
              onClick={() => handleFilterChange(filter.value)}
            >
              {filter.label}
            </button>
          ))}
        </div>
      </div>

      <div>
        <div className="flex items-center gap-2 mb-3">
          <Settings size={16} className={colors.textMuted} />
          <span className={`text-xs font-semibold ${colors.textMuted}`}>操作</span>
        </div>
        <div className="grid grid-cols-2 gap-2">
          <button
            className={`px-3 py-2 border rounded-lg text-xs font-medium transition-all hover:scale-[1.02] flex items-center justify-center gap-1.5 ${colors.text} ${colors.card} ${colors.cardBorder}`}
            onClick={() => setIsSettingsOpen(true)}
          >
            <Settings size={14} />
            <span>设置</span>
          </button>
          <button
            className={`px-3 py-2 border rounded-lg text-xs font-medium transition-all hover:scale-[1.02] flex items-center justify-center gap-1.5 ${colors.text} ${colors.card} ${colors.cardBorder} disabled:opacity-50`}
            onClick={handleExport}
            disabled={isLoading}
          >
            {isLoading ? (
              <Loader2 size={14} className="animate-spin" />
            ) : (
              <Download size={14} />
            )}
            <span>导出</span>
          </button>
          <button
            className="px-3 py-2 border border-cyan-500 text-cyan-500 rounded-lg text-xs font-medium transition-all hover:bg-cyan-500/10 flex items-center justify-center gap-1.5 disabled:opacity-50"
            onClick={handleOpenSyncModal}
            disabled={isSyncing}
          >
            {isSyncing ? (
              <Loader2 size={14} className="animate-spin" />
            ) : (
              <Cloud size={14} />
            )}
            <span>同步到服务器</span>
          </button>
          <button
            className="px-3 py-2 border border-green-500 text-green-500 rounded-lg text-xs font-medium transition-all hover:bg-green-500/10 flex items-center justify-center gap-1.5 col-span-2 disabled:opacity-50"
            onClick={handleImportToMain}
            disabled={isImporting}
          >
            {isImporting ? (
              <Loader2 size={14} className="animate-spin" />
            ) : (
              <Upload size={14} />
            )}
            <span>导入到主账号列表</span>
          </button>
        </div>
      </div>

      {isSettingsOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-[2000] animate-fade-in" onClick={() => setIsSettingsOpen(false)}>
          <div className={`${colors.card} rounded-2xl shadow-xl w-[90%] max-w-[600px] max-h-[90vh] overflow-auto animate-slide-up`} onClick={e => e.stopPropagation()}>
            <div className={`flex items-center justify-between px-6 py-5 border-b ${colors.cardBorder}`}>
              <h2 className={`text-xl font-semibold ${colors.text}`}>系统设置</h2>
              <button
                className={`w-8 h-8 flex items-center justify-center rounded-lg ${isDark ? 'hover:bg-white/10' : 'hover:bg-gray-100'} ${colors.textMuted} text-2xl transition-colors`}
                onClick={() => setIsSettingsOpen(false)}
              >
                ×
              </button>
            </div>
            <div className="p-6">
              <div className="mb-6">
                <h3 className={`text-base font-semibold ${colors.text} mb-2`}>浏览器运行模式</h3>
                <p className={`text-sm ${colors.textMuted} mb-4`}>
                  选择浏览器在注册过程中的显示方式
                </p>
                <div className="space-y-3">
                  <label className={`flex items-start gap-3 cursor-pointer ${isDark ? 'bg-white/5 hover:bg-white/10' : 'bg-gray-50 hover:bg-gray-100'} rounded-xl p-4 transition-all`}>
                    <input
                      type="radio"
                      name="browser_mode"
                      value="background"
                      checked={settings.browser_mode === 'background'}
                      onChange={e =>
                        setSettings({
                          ...settings,
                          browser_mode: e.target.value as 'background' | 'foreground',
                        })
                      }
                      className="mt-0.5 w-4 h-4 rounded-lg border-gray-300 text-blue-500 focus:ring-blue-500"
                    />
                    <div className="flex-1">
                      <div className={`text-sm font-medium ${colors.text}`}>后台运行</div>
                      <div className={`text-xs ${colors.textMuted} mt-1`}>浏览器窗口不可见，在后台执行注册流程</div>
                    </div>
                  </label>
                  <label className={`flex items-start gap-3 cursor-pointer ${isDark ? 'bg-white/5 hover:bg-white/10' : 'bg-gray-50 hover:bg-gray-100'} rounded-xl p-4 transition-all`}>
                    <input
                      type="radio"
                      name="browser_mode"
                      value="foreground"
                      checked={settings.browser_mode === 'foreground'}
                      onChange={e =>
                        setSettings({
                          ...settings,
                          browser_mode: e.target.value as 'background' | 'foreground',
                        })
                      }
                      className="mt-0.5 w-4 h-4 rounded-lg border-gray-300 text-blue-500 focus:ring-blue-500"
                    />
                    <div className="flex-1">
                      <div className={`text-sm font-medium ${colors.text}`}>前台运行</div>
                      <div className={`text-xs ${colors.textMuted} mt-1`}>浏览器窗口可见，可实时观察注册过程</div>
                    </div>
                  </label>
                </div>
              </div>
            </div>
            <div className={`flex items-center justify-end gap-3 px-6 py-4 border-t ${colors.cardBorder}`}>
              <button
                className={`px-5 py-2.5 border rounded-xl font-medium transition-all hover:scale-[1.02] ${colors.text} ${colors.card} ${colors.cardBorder}`}
                onClick={() => setIsSettingsOpen(false)}
              >
                取消
              </button>
              <button
                className="px-5 py-2.5 bg-blue-500 text-white rounded-xl font-medium shadow-sm hover:bg-blue-600 disabled:opacity-50 transition-all flex items-center gap-2"
                onClick={handleSaveSettings}
                disabled={isLoading}
              >
                {isLoading ? (
                  <>
                    <Loader2 size={18} className="animate-spin" />
                    保存中...
                  </>
                ) : (
                  '保存设置'
                )}
              </button>
            </div>
          </div>
        </div>
      )}

      {isSyncModalOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-[2000] animate-fade-in" onClick={() => setIsSyncModalOpen(false)}>
          <div className={`${colors.card} rounded-2xl shadow-xl w-[90%] max-w-[600px] max-h-[90vh] overflow-auto animate-slide-up`} onClick={e => e.stopPropagation()}>
            <div className={`flex items-center justify-between px-6 py-5 border-b ${colors.cardBorder}`}>
              <h2 className={`text-xl font-semibold ${colors.text}`}>同步到服务器</h2>
              <button
                className={`w-8 h-8 flex items-center justify-center rounded-lg ${isDark ? 'hover:bg-white/10' : 'hover:bg-gray-100'} ${colors.textMuted} text-2xl transition-colors`}
                onClick={() => setIsSyncModalOpen(false)}
              >
                ×
              </button>
            </div>
            <div className="p-6">
              <div>
                <h3 className={`text-base font-semibold ${colors.text} mb-2`}>服务器配置</h3>
                <p className={`text-sm ${colors.textMuted} mb-4`}>
                  将已注册的账号同步到闲鱼售卖服务器
                </p>
                <div className="space-y-4">
                  <div>
                    <label htmlFor="serverUrl" className={`block text-sm font-medium ${colors.text} mb-2`}>服务器地址</label>
                    <input
                      type="text"
                      id="serverUrl"
                      value={syncConfig.serverUrl}
                      onChange={e => setSyncConfig({ ...syncConfig, serverUrl: e.target.value })}
                      placeholder="例如: https://your-server.com"
                      className={`w-full px-4 py-3 border rounded-xl ${colors.text} ${colors.input} ${colors.inputFocus} focus:ring-2 transition-all`}
                    />
                  </div>
                  <div>
                    <label htmlFor="secretKey" className={`block text-sm font-medium ${colors.text} mb-2`}>同步密钥</label>
                    <input
                      type="password"
                      id="secretKey"
                      value={syncConfig.secretKey}
                      onChange={e => setSyncConfig({ ...syncConfig, secretKey: e.target.value })}
                      placeholder="请输入同步密钥"
                      className={`w-full px-4 py-3 border rounded-xl ${colors.text} ${colors.input} ${colors.inputFocus} focus:ring-2 transition-all`}
                    />
                  </div>
                </div>
                <p className={`text-xs ${colors.textMuted} mt-3`}>
                  只会同步状态为"已完成"的账号。同步后，账号将可在服务器上通过卡密提取。
                </p>
              </div>
            </div>
            <div className={`flex items-center justify-end gap-3 px-6 py-4 border-t ${colors.cardBorder}`}>
              <button
                className={`px-5 py-2.5 border rounded-xl font-medium transition-all hover:scale-[1.02] ${colors.text} ${colors.card} ${colors.cardBorder}`}
                onClick={() => setIsSyncModalOpen(false)}
              >
                取消
              </button>
              <button
                className="px-5 py-2.5 bg-blue-500 text-white rounded-xl font-medium shadow-sm hover:bg-blue-600 disabled:opacity-50 transition-all flex items-center gap-2"
                onClick={handleSync}
                disabled={isSyncing}
              >
                {isSyncing ? (
                  <>
                    <Loader2 size={18} className="animate-spin" />
                    同步中...
                  </>
                ) : (
                  <>
                    <Cloud size={18} />
                    开始同步
                  </>
                )}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
