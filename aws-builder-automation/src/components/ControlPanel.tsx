import { useState } from 'react';
import { Filter, Trash2, Settings, Loader2, PlayCircle, Download } from 'lucide-react';
import { api } from '../api';
import { useStore } from '../store';
import { showConfirm, showSuccess, showError } from '../utils/dialog';
import './ControlPanel.css';

interface ControlPanelProps {
  onFilterChange: (filter: string | null) => void;
  onRefresh: () => void;
}

export function ControlPanel({ onFilterChange, onRefresh }: ControlPanelProps) {
  const [selectedFilter, setSelectedFilter] = useState<string | null>(null);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const { settings, setSettings } = useStore();
  const [isLoading, setIsLoading] = useState(false);

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
    </div>
  );
}
