import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { 
  Key, Users, Clock, Zap, TrendingUp, Shield, 
  ArrowRight, Sparkles, Activity
} from 'lucide-react'

function Home({ onNavigate }) {
  const [stats, setStats] = useState({
    totalAccounts: 0,
    activeAccounts: 0,
    expiringAccounts: 0
  })
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    loadStats()
  }, [])

  const loadStats = async () => {
    try {
      const accounts = await invoke('get_accounts')
      const now = new Date()
      const oneDay = 24 * 60 * 60 * 1000
      
      setStats({
        totalAccounts: accounts.length,
        activeAccounts: accounts.filter(a => a.status !== '已封禁' && a.status !== '封禁').length,
        expiringAccounts: accounts.filter(a => {
          if (!a.expiresAt) return false
          const exp = new Date(a.expiresAt.replace(/\//g, '-'))
          return exp - now < oneDay && exp > now
        }).length
      })
    } catch (e) {
      console.error(e)
    } finally {
      setLoading(false)
    }
  }

  const statCards = [
    { 
      label: '总账号数', 
      value: stats.totalAccounts, 
      icon: Key, 
      gradient: 'from-blue-500 to-cyan-400',
      bgGradient: 'from-blue-500/20 to-cyan-500/10'
    },
    { 
      label: '活跃账号', 
      value: stats.activeAccounts, 
      icon: Users, 
      gradient: 'from-emerald-500 to-green-400',
      bgGradient: 'from-emerald-500/20 to-green-500/10'
    },
    { 
      label: '即将过期', 
      value: stats.expiringAccounts, 
      icon: Clock, 
      gradient: 'from-amber-500 to-orange-400',
      bgGradient: 'from-amber-500/20 to-orange-500/10'
    },
  ]

  const quickActions = [
    { 
      id: 'token', 
      label: '管理账号', 
      desc: '查看和管理所有账号',
      icon: Key, 
      gradient: 'from-indigo-500 to-purple-500' 
    },
    { 
      id: 'login', 
      label: '桌面授权', 
      desc: '通过桌面客户端授权',
      icon: Shield, 
      gradient: 'from-pink-500 to-rose-500' 
    },
    { 
      id: 'web-oauth', 
      label: '网页授权', 
      desc: '通过浏览器网页授权',
      icon: Zap, 
      gradient: 'from-amber-500 to-orange-500' 
    },
  ]

  return (
    <div className="h-full p-6 overflow-auto">
      <div className="max-w-5xl mx-auto space-y-6">
        {/* 欢迎区 */}
        <div className="glass-card p-8 animate-slide-up">
          <div className="flex items-start justify-between">
            <div>
              <div className="flex items-center gap-3 mb-2">
                <div className="w-12 h-12 rounded-2xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center shadow-lg shadow-indigo-500/30">
                  <Sparkles size={24} className="text-white" />
                </div>
                <div>
                  <h1 className="text-2xl font-bold text-white">欢迎回来</h1>
                  <p className="text-slate-400 text-sm">Kiro Refill Tool</p>
                </div>
              </div>
              <p className="text-slate-500 mt-4 max-w-md">
                高效管理您的 Kiro 账号，支持多账号切换、Token 自动刷新、一键授权等功能。
              </p>
            </div>
            <div className="hidden lg:block">
              <Activity size={120} className="text-indigo-500/20" strokeWidth={1} />
            </div>
          </div>
        </div>

        {/* 统计卡片 */}
        <div className="grid grid-cols-3 gap-4">
          {statCards.map((card, index) => {
            const Icon = card.icon
            return (
              <div 
                key={card.label}
                className="glass-card p-5 animate-slide-up"
                style={{ animationDelay: `${(index + 1) * 100}ms` }}
              >
                <div className="flex items-center justify-between mb-4">
                  <div className={`w-11 h-11 rounded-xl bg-gradient-to-br ${card.bgGradient} flex items-center justify-center`}>
                    <Icon size={20} className={`bg-gradient-to-r ${card.gradient} bg-clip-text text-transparent`} style={{ color: 'currentColor' }} />
                  </div>
                  <TrendingUp size={16} className="text-emerald-400" />
                </div>
                <div className="text-3xl font-bold text-white mb-1">
                  {loading ? <div className="skeleton w-12 h-8" /> : card.value}
                </div>
                <div className="text-sm text-slate-500">{card.label}</div>
              </div>
            )
          })}
        </div>

        {/* 快捷操作 */}
        <div className="animate-slide-up delay-300">
          <h2 className="text-lg font-semibold text-white mb-4">快捷操作</h2>
          <div className="grid grid-cols-3 gap-4">
            {quickActions.map((action, index) => {
              const Icon = action.icon
              return (
                <button
                  key={action.id}
                  onClick={() => onNavigate(action.id)}
                  className="glass-card p-5 text-left group"
                >
                  <div className={`w-12 h-12 rounded-xl bg-gradient-to-br ${action.gradient} flex items-center justify-center mb-4 shadow-lg group-hover:scale-110 transition-transform`}>
                    <Icon size={22} className="text-white" />
                  </div>
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="font-medium text-white mb-1">{action.label}</div>
                      <div className="text-xs text-slate-500">{action.desc}</div>
                    </div>
                    <ArrowRight size={18} className="text-slate-600 group-hover:text-white group-hover:translate-x-1 transition-all" />
                  </div>
                </button>
              )
            })}
          </div>
        </div>

        {/* 提示区 */}
        <div className="glass-card p-5 border-l-4 border-indigo-500 animate-slide-up delay-400">
          <div className="flex items-start gap-4">
            <div className="w-10 h-10 rounded-xl bg-indigo-500/20 flex items-center justify-center flex-shrink-0">
              <Zap size={20} className="text-indigo-400" />
            </div>
            <div>
              <h3 className="font-medium text-white mb-1">小提示</h3>
              <p className="text-sm text-slate-400">
                开启自动刷新功能可以保持 Token 始终有效，避免频繁重新授权。在设置中可以调整刷新间隔。
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default Home
