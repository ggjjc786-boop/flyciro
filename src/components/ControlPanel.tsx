import { useState } from 'react';
import { Filter, Trash2, Settings, Loader2, PlayCircle, Download, Cloud } from 'lucide-react';
import { api } from '../api';
import { useStore } from '../store';
import { showConfirm, showSuccess, showError } from '../utils/dialog';
import './ControlPanel.css';

interface ControlPanelProps {
  onFilterChange: (filter: string | null) => void;
  onRefresh: () => void;
}

// 同步配置（可以保存到本地存储）
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
  const [selectedFilter, setSelectedFilter] = useState<string | null>(null);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isSyncModalOpen, setIsSyncModalOpen] = useState(false);
  const { settings, setSettings } = useStore();
  const [isLoading, setIsLoading] = useState(false);
  const [syncConfig, setSyncConfig] = useState<SyncConfig>(getSyncConfig());
  const [isSyncing, setIsSyncing] = useState(false);

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
    const confirmed = await showConfirm(
      '确定要删除所有账号数据吗（包括进行中的账户）？此操作无法撤销！',
      '确认删除'
    );

    if (confirmed) {
      const doubleConfirmed = await showConfirm(
        '再次确认：这将永久删除所有账号数据（包括进行中状态）！',
        '最终确认'
      );

      if (doubleConfirmed) {
        setIsLoading(true);
        try {
          await api.deleteAllAccounts();
          await showSuccess('已成功删除所有数据');
          onRefresh();
        } catch (error) {
          await showError('删除失败: ' + error);
        } finally {
          setIsLoading(false);
        }
      }
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
    const confirmed = await showConfirm(
      '确定要对所有未注册的账号进行批量注册吗？这将依次处理所有账号。',
      '批量注册确认'
    );

    if (confirmed) {
      setIsLoading(true);
      try {
        await api.startBatchRegistration();
        onRefresh();
      } catch (error) {
        await showError('批量注册失败: ' + error);
      } finally {
        setIsLoading(false);
      }
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

    // 保存配置
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

  return (
    <div className="control-panel">
      <div className="control-section">
        <div className="control-section-header">
          <Filter size={18} />
          <span>状态筛选</span>
        </div>
        <div className="filter-buttons">
          {filters.map(filter => (
            <button
              key={filter.value || 'all'}
              className={`filter-button ${selectedFilter === filter.value ? 'active' : ''}`}
              onClick={() => handleFilterChange(filter.value)}
            >
              {filter.label}
            </button>
          ))}
        </div>
      </div>

      <div className="control-section">
        <div className="control-section-header">
          <Settings size={18} />
          <span>操作</span>
        </div>
        <div className="action-buttons-panel">
          <button
            className="control-action-button"
            onClick={() => setIsSettingsOpen(true)}
          >
            <Settings size={18} />
            系统设置
          </button>
          <button
            className="control-action-button control-action-button-primary"
            onClick={handleBatchRegistration}
            disabled={isLoading}
          >
            {isLoading ? (
              <Loader2 size={18} className="spin" />
            ) : (
              <PlayCircle size={18} />
            )}
            全部注册
          </button>
          <button
            className="control-action-button"
            onClick={handleExport}
            disabled={isLoading}
          >
            {isLoading ? (
              <Loader2 size={18} className="spin" />
            ) : (
              <Download size={18} />
            )}
            导出数据
          </button>
          <button
            className="control-action-button control-action-button-danger"
            onClick={handleDeleteAll}
            disabled={isLoading}
          >
            {isLoading ? (
              <Loader2 size={18} className="spin" />
            ) : (
              <Trash2 size={18} />
            )}
            删除全部
          </button>
          <button
            className="control-action-button control-action-button-sync"
            onClick={handleOpenSyncModal}
            disabled={isSyncing}
          >
            {isSyncing ? (
              <Loader2 size={18} className="spin" />
            ) : (
              <Cloud size={18} />
            )}
            同步到服务器
          </button>
        </div>
      </div>

      {isSettingsOpen && (
        <div className="modal-overlay" onClick={() => setIsSettingsOpen(false)}>
          <div className="modal-content" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h2>系统设置</h2>
              <button
                className="modal-close"
                onClick={() => setIsSettingsOpen(false)}
              >
                ×
              </button>
            </div>
            <div className="modal-body">
              <div className="settings-group">
                <h3>浏览器运行模式</h3>
                <p className="settings-description">
                  选择浏览器在注册过程中的显示方式
                </p>
                <div className="settings-radio-group">
                  <label className="settings-radio-label">
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
                    />
                    <span className="settings-radio-text">
                      <strong>后台运行</strong>
                      <small>浏览器窗口不可见，在后台执行注册流程</small>
                    </span>
                  </label>
                  <label className="settings-radio-label">
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
                    />
                    <span className="settings-radio-text">
                      <strong>前台运行</strong>
                      <small>浏览器窗口可见，可实时观察注册过程</small>
                    </span>
                  </label>
                </div>
              </div>
            </div>
            <div className="modal-footer">
              <button
                className="button-secondary"
                onClick={() => setIsSettingsOpen(false)}
              >
                取消
              </button>
              <button
                className="button-primary"
                onClick={handleSaveSettings}
                disabled={isLoading}
              >
                {isLoading ? (
                  <>
                    <Loader2 size={18} className="spin" />
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

      {/* 同步配置模态框 */}
      {isSyncModalOpen && (
        <div className="modal-overlay" onClick={() => setIsSyncModalOpen(false)}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h2>同步到服务器</h2>
              <button
                className="modal-close-button"
                onClick={() => setIsSyncModalOpen(false)}
              >
                ×
              </button>
            </div>
            <div className="modal-body">
              <div className="settings-group">
                <h3>服务器配置</h3>
                <p className="settings-description">
                  将已注册的账号同步到闲鱼售卖服务器
                </p>
                <div className="sync-form">
                  <div className="form-field">
                    <label htmlFor="serverUrl">服务器地址</label>
                    <input
                      type="text"
                      id="serverUrl"
                      value={syncConfig.serverUrl}
                      onChange={e => setSyncConfig({ ...syncConfig, serverUrl: e.target.value })}
                      placeholder="例如: https://your-server.com"
                    />
                  </div>
                  <div className="form-field">
                    <label htmlFor="secretKey">同步密钥</label>
                    <input
                      type="password"
                      id="secretKey"
                      value={syncConfig.secretKey}
                      onChange={e => setSyncConfig({ ...syncConfig, secretKey: e.target.value })}
                      placeholder="请输入同步密钥"
                    />
                  </div>
                </div>
                <p className="settings-hint" style={{ marginTop: '10px', color: '#888', fontSize: '12px' }}>
                  只会同步状态为"已完成"的账号。同步后，账号将可在服务器上通过卡密提取。
                </p>
              </div>
            </div>
            <div className="modal-footer">
              <button
                className="button-secondary"
                onClick={() => setIsSyncModalOpen(false)}
              >
                取消
              </button>
              <button
                className="button-primary"
                onClick={handleSync}
                disabled={isSyncing}
              >
                {isSyncing ? (
                  <>
                    <Loader2 size={18} className="spin" />
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
