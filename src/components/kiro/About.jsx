import { useState, useEffect } from 'react'
import { getVersion } from '@tauri-apps/api/app'
import { 
  Heart, Sparkles, Code, Coffee
} from 'lucide-react'

function About() {
  const [version, setVersion] = useState('')

  useEffect(() => {
    getVersion().then(setVersion)
  }, [])

  const features = [
    '多账号管理与快速切换',
    'Token 自动刷新保活',
    '桌面/网页双授权方式',
    '自动注册新账号',
    'Kiro IDE 配置同步',
  ]

  return (
    <div className="h-full p-6 overflow-auto">
      <div className="max-w-2xl mx-auto space-y-6">
        {/* Logo 区域 */}
        <div className="glass-card p-8 text-center animate-slide-up">
          <div className="w-20 h-20 bg-gradient-to-br from-indigo-500 to-purple-600 rounded-3xl flex items-center justify-center mx-auto mb-6 shadow-2xl shadow-indigo-500/30">
            <Sparkles size={40} className="text-white" />
          </div>
          <h1 className="text-3xl font-bold text-white mb-2">Kiro Refill Tool</h1>
          <p className="text-slate-400 mb-4">高效的 Kiro 账号管理工具</p>
          <div className="badge badge-aurora">
            v{version || '...'}
          </div>
        </div>

        {/* 功能特性 */}
        <div className="glass-card p-6 animate-slide-up delay-100">
          <div className="flex items-center gap-3 mb-4">
            <Code size={20} className="text-emerald-400" />
            <h2 className="text-lg font-semibold text-white">功能特性</h2>
          </div>
          <div className="grid grid-cols-1 gap-2">
            {features.map((feature, index) => (
              <div 
                key={index}
                className="flex items-center gap-3 p-3 rounded-xl bg-white/5"
              >
                <div className="w-2 h-2 rounded-full bg-gradient-to-r from-indigo-500 to-purple-500" />
                <span className="text-slate-300 text-sm">{feature}</span>
              </div>
            ))}
          </div>
        </div>

        {/* 致谢 */}
        <div className="glass-card p-6 animate-slide-up delay-200">
          <div className="flex items-center gap-3 mb-4">
            <Heart size={20} className="text-pink-400" />
            <h2 className="text-lg font-semibold text-white">致谢</h2>
          </div>
          <p className="text-slate-400 text-sm leading-relaxed">
            感谢所有用户的支持与反馈，让这个工具变得更好。
            本项目基于 Tauri + React + TailwindCSS 构建。
          </p>
        </div>

        {/* 版权 */}
        <div className="text-center py-4 animate-slide-up delay-300">
          <div className="flex items-center justify-center gap-2 text-slate-600 text-sm">
            <Coffee size={14} />
            <span>Made with love</span>
          </div>
          <div className="text-slate-700 text-xs mt-2">
            © 2024-2026 Kiro Refill Tool. All rights reserved.
          </div>
        </div>
      </div>
    </div>
  )
}

export default About
