import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'
import { 
  LayoutDashboard, Key, UserPlus, Wrench, LogIn, Globe, 
  Settings, Info, Crown, LogOut, ChevronRight
} from 'lucide-react'
import { useI18n } from '../i18n.jsx'

const menuItems = [
  { id: 'home', icon: LayoutDashboard, label: '仪表盘', gradient: 'from-blue-500 to-cyan-400' },
  { id: 'token', icon: Key, label: '账号管理', gradient: 'from-amber-500 to-orange-400' },
  { id: 'register', icon: UserPlus, label: '注册账号', gradient: 'from-emerald-500 to-green-400', vipOnly: true },
  { id: 'kiro-config', icon: Wrench, label: 'Kiro 配置', gradient: 'from-violet-500 to-purple-400' },
  { id: 'login', icon: LogIn, label: '桌面授权', gradient: 'from-pink-500 to-rose-400' },
  { id: 'web-oauth', icon: Globe, label: '网页授权', gradient: 'from-indigo-500 to-blue-400' },
]

const bottomItems = [
  { id: 'settings', icon: Settings, label: '设置' },
  { id: 'about', icon: Info, label: '关于' },
]

function Sidebar({ activeMenu, onMenuChange, userInfo, onUserLogout }) {
  const [version, setVersion] = useState('')
  const [localToken, setLocalToken] = useState(null)
  const { t } = useI18n()
  
  const isVip = userInfo?.is_vip || false

  useEffect(() => {
    getVersion().then(setVersion)
    invoke('get_kiro_local_token').then(setLocalToken).catch(() => {})
  }, [])

  return (
    <div className="w-64 glass-panel border-t-0 border-l-0 flex flex-col">
      {/* 用户信息区 */}
      <div className="p-4 border-b border-white/5">
        {userInfo ? (
          <div className="flex items-center gap-3">
            <div className={`w-11 h-11 rounded-xl flex items-center justify-center text-white font-bold shadow-lg ${
              isVip 
                ? 'bg-gradient-to-br from-amber-400 to-orange-500' 
                : 'bg-gradient-to-br from-slate-500 to-slate-600'
            }`}>
              {userInfo.nickname?.[0] || userInfo.email?.[0]?.toUpperCase() || userInfo.phone?.slice(-2) || 'U'}
            </div>
            <div className="flex-1 min-w-0">
              <div className="text-sm font-medium text-white truncate">
                {userInfo.nickname || userInfo.email || userInfo.phone}
              </div>
              <div className={`text-xs flex items-center gap-1 ${isVip ? 'text-amber-400' : 'text-slate-500'}`}>
                <Crown size={12} />
                {isVip ? 'VIP 会员' : '普通用户'}
              </div>
            </div>
            <button
              onClick={onUserLogout}
              className="p-2 rounded-lg hover:bg-red-500/20 transition-colors group"
              title="退出登录"
            >
              <LogOut size={16} className="text-slate-500 group-hover:text-red-400" />
            </button>
          </div>
        ) : (
          <button
            onClick={() => onMenuChange('user-auth')}
            className="w-full flex items-center gap-3 p-3 rounded-xl bg-gradient-to-r from-indigo-500/20 to-purple-500/20 border border-indigo-500/30 hover:from-indigo-500/30 hover:to-purple-500/30 transition-all group"
          >
            <div className="w-9 h-9 rounded-lg bg-gradient-to-br from-indigo-500 to-purple-500 flex items-center justify-center">
              <Crown size={18} className="text-white" />
            </div>
            <div className="flex-1 text-left">
              <div className="text-sm font-medium text-white">登录 / 注册</div>
              <div className="text-xs text-slate-400">开通会员解锁更多功能</div>
            </div>
            <ChevronRight size={16} className="text-slate-500 group-hover:text-white transition-colors" />
          </button>
        )}
      </div>

      {/* 主导航 */}
      <nav className="flex-1 p-3 space-y-1 overflow-y-auto">
        {menuItems.map((item, index) => {
          const Icon = item.icon
          const isActive = activeMenu === item.id
          const isLocked = item.vipOnly && !isVip
          
          return (
            <button
              key={item.id}
              onClick={() => !isLocked && onMenuChange(item.id)}
              disabled={isLocked}
              className={`nav-item w-full animate-slide-left ${isActive ? 'active' : ''} ${isLocked ? 'opacity-40 cursor-not-allowed' : ''}`}
              style={{ animationDelay: `${index * 50}ms` }}
            >
              <div className={`w-9 h-9 rounded-xl flex items-center justify-center transition-all ${
                isActive 
                  ? `bg-gradient-to-br ${item.gradient} shadow-lg` 
                  : 'bg-white/5'
              }`}>
                <Icon size={18} className={isActive ? 'text-white' : 'text-slate-400'} />
              </div>
              <span className="flex-1 text-left text-sm">{item.label}</span>
              {item.vipOnly && (
                <Crown size={14} className={isVip ? 'text-amber-400' : 'text-slate-600'} />
              )}
            </button>
          )
        })}
      </nav>

      {/* Kiro 连接状态 */}
      {localToken && (
        <div className="mx-3 mb-3 p-3 rounded-xl bg-emerald-500/10 border border-emerald-500/20">
          <div className="flex items-center gap-2">
            <div className="status-indicator status-online" />
            <span className="text-xs text-emerald-400">Kiro IDE 已连接</span>
          </div>
        </div>
      )}

      {/* 底部导航 */}
      <div className="p-3 border-t border-white/5 space-y-1">
        {bottomItems.map((item) => {
          const Icon = item.icon
          const isActive = activeMenu === item.id
          
          return (
            <button
              key={item.id}
              onClick={() => onMenuChange(item.id)}
              className={`nav-item w-full ${isActive ? 'active' : ''}`}
            >
              <Icon size={18} className={isActive ? 'text-white' : 'text-slate-500'} />
              <span className="flex-1 text-left text-sm">{item.label}</span>
            </button>
          )
        })}
        
        {/* 版本号 */}
        <div className="pt-2 text-center">
          <span className="text-[10px] text-slate-600">v{version || '...'}</span>
        </div>
      </div>
    </div>
  )
}

export default Sidebar
