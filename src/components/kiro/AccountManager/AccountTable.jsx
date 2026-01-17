import { Users, Plus, CheckSquare, Square } from 'lucide-react'
import { useTheme } from '../../contexts/ThemeContext'
import { useI18n } from '../../i18n.jsx'
import AccountCard from './AccountCard'

function AccountTable({
  accounts,
  filteredAccounts,
  selectedIds,
  onSelectAll,
  onSelectOne,
  copiedId,
  onCopy,
  onSwitch,
  onRefresh,
  onEdit,
  onEditLabel,
  onDelete,
  onAdd,
  refreshingId,
  switchingId,
  localToken,
}) {
  const { colors, isDark } = useTheme()
  const { t } = useI18n()

  const allSelected = selectedIds.length === filteredAccounts.length && filteredAccounts.length > 0

  return (
    <div className="flex-1 overflow-auto p-6">
      {/* Select All Control */}
      {accounts.length > 0 && (
        <div className="flex items-center gap-3 mb-5 animate-fade-in">
          <button
            onClick={() => onSelectAll(!allSelected)}
            className={`
              flex items-center gap-2 px-3 py-2 rounded-xl transition-all
              ${isDark ? 'hover:bg-white/5' : 'hover:bg-gray-100'}
            `}
          >
            {allSelected ? (
              <CheckSquare size={18} className="text-indigo-500" />
            ) : (
              <Square size={18} className={colors.textMuted} />
            )}
            <span className={`text-sm ${selectedIds.length > 0 ? colors.text : colors.textMuted}`}>
              {selectedIds.length > 0 
                ? `${t('common.selected')} ${selectedIds.length}` 
                : t('common.selectAll')
              }
            </span>
          </button>
        </div>
      )}

      {/* Cards Grid */}
      {accounts.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-24 animate-fade-in">
          <div className={`
            w-24 h-24 rounded-2xl flex items-center justify-center mb-6 animate-float
            ${isDark ? 'bg-white/5' : 'bg-gray-100'}
          `}>
            <Users size={48} strokeWidth={1.5} className={colors.textMuted} />
          </div>
          <p className={`font-semibold ${colors.text} mb-2`}>{t('common.noAccounts')}</p>
          <p className={`text-sm ${colors.textMuted} mb-6`}>{t('common.addAccountHint')}</p>
          <button onClick={onAdd} className="btn btn-primary">
            <Plus size={18} />
            {t('common.addAccount')}
          </button>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-5">
          {accounts.map((account, index) => (
            <div 
              key={account.id} 
              className="animate-fade-in-up"
              style={{ animationDelay: `${index * 30}ms` }}
            >
              <AccountCard
                account={account}
                isSelected={selectedIds.includes(account.id)}
                onSelect={(checked) => onSelectOne(account.id, checked)}
                copiedId={copiedId}
                onCopy={onCopy}
                onSwitch={onSwitch}
                onRefresh={onRefresh}
                onEdit={onEdit}
                onEditLabel={onEditLabel}
                onDelete={onDelete}
                refreshingId={refreshingId}
                switchingId={switchingId}
                isCurrentAccount={localToken?.refreshToken && account.refreshToken === localToken.refreshToken}
              />
            </div>
          ))}
          
          {/* Add Account Card */}
          <button
            onClick={onAdd}
            className={`
              rounded-2xl border-2 border-dashed transition-all duration-300 min-h-[300px]
              flex flex-col items-center justify-center gap-4 group
              animate-fade-in-up hover-card
              ${isDark 
                ? 'border-white/10 hover:border-indigo-500/50 hover:bg-indigo-500/5' 
                : 'border-gray-200 hover:border-indigo-400 hover:bg-indigo-50/50'
              }
            `}
            style={{ animationDelay: `${accounts.length * 30}ms` }}
          >
            <div className={`
              w-14 h-14 rounded-2xl flex items-center justify-center transition-all duration-300
              ${isDark 
                ? 'bg-white/5 group-hover:bg-indigo-500/20' 
                : 'bg-gray-100 group-hover:bg-indigo-100'
              }
            `}>
              <Plus size={28} className={`${colors.textMuted} group-hover:text-indigo-500 transition-colors`} />
            </div>
            <span className={`text-sm font-medium ${colors.textMuted} group-hover:text-indigo-500 transition-colors`}>
              {t('common.addAccount')}
            </span>
          </button>
        </div>
      )}
    </div>
  )
}

export default AccountTable
