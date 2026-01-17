import { Search, Download, Upload, RefreshCw, Trash2, Plus, Key } from 'lucide-react'
import { useTheme } from '../../contexts/ThemeContext'
import { useI18n } from '../../i18n.jsx'

function AccountHeader({
  searchTerm,
  onSearchChange,
  selectedCount,
  onBatchDelete,
  onAdd,
  onImport,
  onExport,
  onRefreshAll,
  autoRefreshing,
  lastRefreshTime,
  refreshProgress,
}) {
  const { colors, isDark } = useTheme()
  const { t } = useI18n()

  return (
    <div className={`px-6 py-5 border-b ${colors.cardBorder} relative overflow-hidden`}>
      {/* Background Decoration */}
      <div className="absolute top-0 right-0 w-96 h-96 bg-gradient-to-br from-indigo-500/5 to-purple-500/5 rounded-full blur-3xl -translate-y-1/2 translate-x-1/2 pointer-events-none" />
      
      <div className="flex items-center justify-between relative">
        {/* Title Section */}
        <div className="flex items-center gap-4 animate-fade-in-down">
          <div className="w-12 h-12 bg-gradient-to-br from-amber-500 to-orange-500 rounded-2xl flex items-center justify-center shadow-xl shadow-amber-500/30 animate-float">
            <Key size={24} className="text-white" />
          </div>
          <div>
            <h1 className={`text-2xl font-bold ${colors.text}`}>{t('accounts.title')}</h1>
            <p className={`text-sm ${colors.textMuted} mt-0.5`}>{t('accounts.subtitle')}</p>
          </div>
        </div>

        {/* Actions Section */}
        <div className="flex items-center gap-3 animate-fade-in">
          {/* Last Refresh Time */}
          {lastRefreshTime && !autoRefreshing && (
            <span className={`text-xs ${colors.textMuted} px-3 py-1.5 rounded-lg ${isDark ? 'bg-white/5' : 'bg-gray-100'}`}>
              {lastRefreshTime}
            </span>
          )}
          
          {/* Search Input */}
          <div className="relative group">
            <Search className={`absolute left-3 top-1/2 -translate-y-1/2 ${colors.textMuted} transition-colors group-focus-within:text-indigo-500`} size={16} />
            <input
              type="text"
              placeholder={t('accounts.search')}
              value={searchTerm}
              onChange={(e) => onSearchChange(e.target.value)}
              className={`
                input pl-10 pr-4 py-2.5 w-52 text-sm rounded-xl
                ${isDark ? 'bg-white/5 border-white/10' : 'bg-gray-50 border-gray-200'}
                focus:w-64 transition-all duration-300
              `}
            />
          </div>
          
          {/* Batch Delete */}
          {selectedCount > 0 && (
            <button 
              onClick={onBatchDelete} 
              className="btn btn-danger text-sm animate-scale-in"
            >
              <Trash2 size={16} />
              {t('common.delete')} ({selectedCount})
            </button>
          )}
          
          {/* Add Button */}
          <button 
            onClick={onAdd} 
            className="btn btn-primary text-sm"
          >
            <Plus size={16} />
            {t('common.add')}
          </button>
          
          {/* Import Button */}
          <button 
            onClick={onImport} 
            className={`btn ${colors.btnSecondary} text-sm`}
            title={t('accounts.import')}
          >
            <Upload size={16} />
            {t('accounts.import')}
          </button>
          
          {/* Export Button */}
          <button 
            onClick={onExport} 
            className={`btn ${colors.btnSecondary} text-sm`}
            title={t('accounts.export')}
          >
            <Download size={16} />
            {t('accounts.export')}
          </button>
          
          {/* Refresh All Button */}
          <button 
            onClick={onRefreshAll} 
            disabled={autoRefreshing} 
            className={`
              p-2.5 rounded-xl transition-all
              ${isDark ? 'bg-white/5 hover:bg-white/10 border border-white/10' : 'bg-gray-100 hover:bg-gray-200 border border-gray-200'}
              disabled:opacity-50 disabled:cursor-not-allowed
            `}
            title={t('accounts.refreshAll')}
          >
            <RefreshCw size={18} className={`${colors.textMuted} ${autoRefreshing ? 'animate-spin' : ''}`} />
          </button>
        </div>
      </div>
      
      {/* Progress Bar */}
      {autoRefreshing && refreshProgress.total > 0 && (
        <div className="mt-4 flex items-center gap-4 animate-fade-in">
          <div className="flex-1 progress-bar h-2">
            <div 
              className="progress-fill progress-fill-primary" 
              style={{ width: `${(refreshProgress.current / refreshProgress.total) * 100}%` }} 
            />
          </div>
          <span className={`text-xs font-medium ${isDark ? 'text-indigo-400' : 'text-indigo-600'}`}>
            {refreshProgress.current}/{refreshProgress.total}
          </span>
        </div>
      )}
    </div>
  )
}

export default AccountHeader
