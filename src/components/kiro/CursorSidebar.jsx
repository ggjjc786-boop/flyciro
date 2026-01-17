import { useState, useEffect } from 'react'
import { getVersion } from '@tauri-apps/api/app'
import { 
  LayoutDashboard, Key, UserPlus, RefreshCw,
  Settings, Info, MousePointer2
} from 'lucide-react'

const menuItems = [
  { id: 'cursor-home', icon: LayoutDashboard, label: '仪表盘', gradient: 'from-cyan-500 to-blue-400' },
  { id: 'cursor-accounts', icon: Key, label: '账号管理', gradient: 'from-emerald-500 to-green-400' },
  { id: 'cursor-register', icon: UserPlus, label: '自动注册', gradient: 'from-violet-500 to-purple-400' },
  { id: 'cursor-reset', icon: RefreshCw, label: '重置机器码', gradient: 'from-amber-500 to-orange-400' },
]

const bottomItems = [
  { id: 'cursor-settings', icon: Settings, label: '设置' },
  { id: 'cursor-about', icon: Info, label: '关于' },
]

function CursorSidebar({ activeMenu, onMenuChange }) {
  const [version, setVersion] = useState('')

  useEffect(() => {
    getVersion().then(setVersion)
  }, [])

  return (
    <div className="w-64 glass-panel border-t-0 border-l-0 flex flex-col">
      {/* Logo 区域 */}
      <div className="p-4 border-b border-white/5">
        <div className="flex items-center gap-3">
          <div className="w-11 h-11 rounded-xl bg-gradient-to-br from-cyan-500 to-blue-600 flex items-center justify-center shadow-lg shadow-cyan-500/30">
            <MousePointer2 size={22} className="text-white" />
          </div>
          <div>
            <div className="text-sm font-semibold text-white">Cursor 工具</div>
            <div className="text-xs text-slate-500">自动注册与管理</div>
          </div>
        </div>
      </div>

      {/* 主导航 */}
      <nav className="flex-1 p-3 space-y-1 overflow-y-auto">
        {menuItems.map((item, index) => {
          const Icon = item.icon
          const isActive = activeMenu === item.id
          
          return (
            <button
              key={item.id}
              onClick={() => onMenuChange(item.id)}
              className={`nav-item w-full animate-slide-left ${isActive ? 'active' : ''}`}
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
            </button>
          )
        })}
      </nav>

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

export default CursorSidebar
