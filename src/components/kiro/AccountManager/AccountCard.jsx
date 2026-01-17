import { RefreshCw, Eye, Trash2, Copy, Check, Clock, Repeat, Edit2, Zap, CheckCircle2 } from 'lucide-react'
import { useTheme } from '../../contexts/ThemeContext'
import { useI18n } from '../../i18n.jsx'
import { getUsagePercent, getProgressBarColor } from './hooks/useAccountStats'
import { getQuota, getUsed, getSubType, getSubPlan } from '../../utils/accountStats'

function AccountCard({
  account,
  isSelected,
  onSelect,
  copiedId,
  onCopy,
  onSwitch,
  onRefresh,
  onEdit,
  onEditLabel,
  onDelete,
  refreshingId,
  switchingId,
  isCurrentAccount,
}) {
  const { colors, isDark } = useTheme()
  const { t } = useI18n()
  
  const quota = getQuota(account)
  const used = getUsed(account)
  const subType = getSubType(account)
  const subPlan = getSubPlan(account)
  const breakdown = account.usageData?.usageBreakdownList?.[0]
  const percent = getUsagePercent(used, quota)
  const isExpired = account.expiresAt && new Date(account.expiresAt.replace(/\//g, '-')) < new Date()
  const isBanned = account.status === '封禁' || account.status === '已封禁'
  const isNormal = account.status === '正常' || account.status === '有效'

  // Card border and background based on status
  const getCardStyle = () => {
    if (isSelected) {
      return isDark 
        ? 'border-indigo-500 bg-indigo-500/10 shadow-lg shadow-indigo-500/20' 
        : 'border-indigo-400 bg-indigo-50 shadow-lg shadow-indigo-500/10'
    }
    if (isCurrentAccount) {
      return isDark 
        ? 'border-emerald-500/50 bg-emerald-500/5 shadow-lg shadow-emerald-500/20' 
        : 'border-emerald-400 bg-emerald-50/50 shadow-lg shadow-emerald-500/10'
    }
    if (isBanned) {
      return isDark 
        ? 'border-red-500/50 bg-red-500/5' 
        : 'border-red-300 bg-red-50/50'
    }
    if (!isNormal) {
      return isDark 
        ? 'border-amber-500/50 bg-amber-500/5' 
        : 'border-amber-300 bg-amber-50/50'
    }
    return isDark 
      ? 'border-white/10 bg-white/5 hover:border-white/20 hover:bg-white/8' 
      : 'border-gray-200 bg-white hover:border-gray-300 hover:shadow-md'
  }

  // Avatar gradient based on provider
  const getAvatarStyle = () => {
    switch (account.provider) {
      case 'Google':
        return 'bg-gradient-to-br from-red-500 to-orange-500 shadow-red-500/30'
      case 'Github':
        return 'bg-gradient-to-br from-gray-700 to-gray-900 shadow-gray-500/30'
      default:
        return 'bg-gradient-to-br from-indigo-500 to-purple-500 shadow-indigo-500/30'
    }
  }

  return (
    <div className={`
      relative rounded-2xl border transition-all duration-300 flex flex-col
      hover-card ${getCardStyle()}
    `}>
      {/* Selection Checkbox */}
      <div className="absolute top-4 left-4 z-10">
        <label className="cursor-pointer">
          <input 
            type="checkbox" 
            checked={isSelected} 
            onChange={(e) => onSelect(e.target.checked)} 
            className="sr-only peer" 
          />
          <div className={`
            w-5 h-5 rounded-lg border-2 flex items-center justify-center transition-all
            peer-checked:bg-indigo-500 peer-checked:border-indigo-500
            ${isDark ? 'border-white/20 bg-white/5' : 'border-gray-300 bg-white'}
          `}>
            {isSelected && <Check size={12} className="text-white" />}
          </div>
        </label>
      </div>
      
      {/* Status Badge */}
      <div className="absolute top-4 right-4 flex items-center gap-2">
        {isCurrentAccount && (
          <span className="badge badge-success flex items-center gap-1 text-[10px]">
            <Zap size={10} />
            {t('common.currentlyUsing')}
          </span>
        )}
        <span className={`badge text-[10px] ${
          isNormal ? 'badge-success' : isBanned ? 'badge-danger' : 'badge-warning'
        }`}>
          {account.status}
        </span>
      </div>

      <div className="p-5 pt-12 flex-1 flex flex-col">
        {/* Avatar and Email */}
        <div className="flex items-start gap-4 mb-4">
          <div className={`
            w-12 h-12 rounded-xl flex items-center justify-center text-white font-bold text-lg shadow-lg
            ${getAvatarStyle()}
          `}>
            {account.email[0].toUpperCase()}
          </div>
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2">
              <span className={`font-semibold ${colors.text} text-sm truncate`}>{account.email}</span>
              <button 
                onClick={() => onCopy(account.email, account.id)} 
                className={`p-1 rounded-lg transition-all ${isDark ? 'hover:bg-white/10' : 'hover:bg-gray-100'}`}
              >
                {copiedId === account.id 
                  ? <CheckCircle2 size={14} className="text-emerald-500" /> 
                  : <Copy size={14} className={colors.textMuted} />
                }
              </button>
            </div>
            <div className={`text-xs ${colors.textMuted} mt-0.5`}>
              {account.label || account.provider || t('common.noLabel')}
            </div>
          </div>
        </div>

        {/* Subscription Type */}
        <div className="flex items-center gap-2 mb-4">
          <span className={`
            px-2.5 py-1 rounded-lg text-xs font-semibold
            ${(subType.includes('PRO+') || subPlan.includes('PRO+'))
              ? 'bg-gradient-to-r from-purple-500 to-pink-500 text-white shadow-md shadow-purple-500/30'
              : (subType.includes('PRO') || subPlan.includes('PRO'))
                ? 'bg-gradient-to-r from-blue-500 to-indigo-500 text-white shadow-md shadow-blue-500/30'
                : isDark ? 'bg-white/10 text-white/70' : 'bg-gray-100 text-gray-600'
            }
          `}>
            {subPlan || 'Free'}
          </span>
          <span className={`text-xs px-2 py-1 rounded-lg ${isDark ? 'bg-white/5 text-white/50' : 'bg-gray-50 text-gray-500'}`}>
            {account.provider || t('common.unknown')}
          </span>
        </div>

        {/* Usage Progress */}
        <div className={`p-4 rounded-xl mb-4 ${isDark ? 'bg-white/5' : 'bg-gray-50'}`}>
          <div className="flex items-center justify-between text-xs mb-2">
            <span className={colors.textMuted}>{t('common.usage')}</span>
            <span className={`font-bold ${
              percent > 80 ? 'text-red-500' : percent > 50 ? 'text-amber-500' : 'text-emerald-500'
            }`}>
              {Math.round(percent)}%
            </span>
          </div>
          <div className="progress-bar h-2 mb-2">
            <div 
              className={`progress-fill ${
                percent > 80 ? 'progress-fill-danger' : 
                percent > 50 ? 'progress-fill-warning' : 
                'progress-fill-success'
              }`}
              style={{ width: `${Math.min(percent, 100)}%` }} 
            />
          </div>
          <div className="flex items-center justify-between text-xs">
            <span className={colors.text}>{Math.round(used * 100) / 100} / {quota}</span>
            <span className={colors.textMuted}>{t('common.remaining')} {Math.round((quota - used) * 100) / 100}</span>
          </div>
          {breakdown?.nextDateReset && (
            <div className={`text-[10px] ${colors.textMuted} mt-2 flex items-center gap-1`}>
              <Clock size={10} />
              {new Date(breakdown.nextDateReset * 1000).toLocaleDateString()} {t('common.reset')}
            </div>
          )}
        </div>

        {/* Token Expiry */}
        {account.expiresAt && (
          <div className={`text-xs mb-4 flex items-center gap-1.5 ${isExpired ? 'text-red-500' : colors.textMuted}`}>
            <Clock size={12} />
            <span>Token: {account.expiresAt}</span>
            {isExpired && <span className="font-semibold">({t('accountCard.tokenExpired')})</span>}
          </div>
        )}

        {/* Action Buttons */}
        <div className={`flex items-center justify-center gap-1 pt-4 mt-auto border-t ${colors.cardBorder}`}>
          <ActionButton 
            onClick={() => onSwitch(account)} 
            disabled={switchingId === account.id}
            icon={Repeat}
            iconClass={`text-indigo-500 ${switchingId === account.id ? 'animate-spin' : ''}`}
            title={t('accountCard.switchAccount')}
            isDark={isDark}
          />
          <ActionButton 
            onClick={() => onRefresh(account.id)} 
            disabled={refreshingId === account.id}
            icon={RefreshCw}
            iconClass={`${colors.textMuted} ${refreshingId === account.id ? 'animate-spin' : ''}`}
            title={t('accountCard.refresh')}
            isDark={isDark}
          />
          <ActionButton 
            onClick={() => onEdit(account)}
            icon={Eye}
            iconClass={colors.textMuted}
            title={t('accountCard.viewDetails')}
            isDark={isDark}
          />
          <ActionButton 
            onClick={() => onEditLabel(account)}
            icon={Edit2}
            iconClass={colors.textMuted}
            title={t('accountCard.editRemark')}
            isDark={isDark}
          />
          <ActionButton 
            onClick={() => onDelete(account.id)}
            icon={Trash2}
            iconClass="text-red-400"
            hoverClass={isDark ? 'hover:bg-red-500/20' : 'hover:bg-red-50'}
            title={t('accountCard.delete')}
            isDark={isDark}
          />
        </div>
      </div>
    </div>
  )
}

function ActionButton({ onClick, disabled, icon: Icon, iconClass, hoverClass, title, isDark }) {
  return (
    <button 
      onClick={onClick} 
      disabled={disabled}
      className={`
        p-2.5 rounded-xl transition-all disabled:opacity-50 disabled:cursor-not-allowed
        ${hoverClass || (isDark ? 'hover:bg-white/10' : 'hover:bg-gray-100')}
      `}
      title={title}
    >
      <Icon size={16} className={iconClass} />
    </button>
  )
}

export default AccountCard
