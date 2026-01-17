import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { emit } from '@tauri-apps/api/event'
import { 
  Settings as SettingsIcon, RefreshCw, Clock, Monitor, 
  Save, RotateCcw, CheckCircle, Sun, Moon
} from 'lucide-react'
import { useTheme, themes } from '../contexts/ThemeContext'
import { useI18n, locales } from '../i18n.jsx'

function Settings() {
  const { theme, setTheme } = useTheme()
  const { locale, setLocale, t } = useI18n()
  
  const [settings, setSettings] = useState({
    autoRefresh: true,
    autoRefreshInterval: 50,
    browserPath: '',
    machineGuid: ''
  })
  const [saving, setSaving] = useState(false)
  const [saved, setSaved] = useState(false)

  useEffect(() => {
    loadSettings()
  }, [])

  const loadSettings = async () => {
    try {
      const data = await invoke('get_app_settings')
      setSettings(prev => ({ ...prev, ...data }))
    } catch (e) {
      console.error('Failed to load settings:', e)
    }
  }

  const handleSave = async () => {
    setSaving(true)
    try {
      await invoke('save_app_settings', { settings })
      await emit('settings-changed')
      setSaved(true)
      setTimeout(() => setSaved(false), 2000)
    } catch (e) {
      console.error('Failed to save settings:', e)
    } finally {
      setSaving(false)
    }
  }

  const handleReset = () => {
    setSettings({
      autoRefresh: true,
      autoRefreshInterval: 50,
      browserPath: '',
      machineGuid: ''
    })
  }

  return (
    <div className="h-full p-6 overflow-auto">
      <div className="max-w-3xl mx-auto space-y-6">
        {/* 标题 */}
        <div className="flex items-center gap-3 animate-slide-up">
          <div className="w-12 h-12 rounded-2xl bg-gradient-to-br from-slate-500 to-slate-600 flex items-center justify-center shadow-lg">
            <SettingsIcon size={24} className="text-white" />
          </div>
          <div>
            <h1 className="text-2xl font-bold text-white">{t('settings.title')}</h1>
            <p className="text-slate-500 text-sm">{t('settings.subtitle')}</p>
          </div>
        </div>

        {/* 自动刷新设置 */}
        <div className="glass-card p-6 animate-slide-up delay-100">
          <div className="flex items-center gap-3 mb-6">
            <RefreshCw size={20} className="text-indigo-400" />
            <h2 className="text-lg font-semibold text-white">{t('settings.autoRefresh')}</h2>
          </div>
          
          <div className="space-y-5">
            <div className="flex items-center justify-between">
              <div>
                <div className="text-white font-medium">{t('settings.enableAutoRefresh')}</div>
                <div className="text-sm text-slate-500">{t('settings.autoRefreshDesc')}</div>
              </div>
              <button
                onClick={() => setSettings(s => ({ ...s, autoRefresh: !s.autoRefresh }))}
                className={`w-14 h-8 rounded-full transition-all relative ${
                  settings.autoRefresh 
                    ? 'bg-indigo-500' 
                    : 'bg-slate-700'
                }`}
              >
                <div className={`w-6 h-6 rounded-full bg-white shadow-md absolute top-1 transition-all ${
                  settings.autoRefresh ? 'left-7' : 'left-1'
                }`} />
              </button>
            </div>
            
            {settings.autoRefresh && (
              <div className="pt-4 border-t border-white/5">
                <div className="flex items-center gap-3 mb-3">
                  <Clock size={16} className="text-slate-500" />
                  <span className="text-sm text-slate-400">{t('settings.refreshInterval')}</span>
                </div>
                <div className="flex items-center gap-4">
                  <input
                    type="range"
                    min="10"
                    max="120"
                    value={settings.autoRefreshInterval}
                    onChange={(e) => setSettings(s => ({ ...s, autoRefreshInterval: parseInt(e.target.value) }))}
                    className="flex-1 h-2 bg-slate-700 rounded-full appearance-none cursor-pointer accent-indigo-500"
                  />
                  <span className="text-white font-mono w-20 text-right">
                    {settings.autoRefreshInterval} {t('settings.minutes')}
                  </span>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* 外观设置 */}
        <div className="glass-card p-6 animate-slide-up delay-200">
          <div className="flex items-center gap-3 mb-6">
            <Monitor size={20} className="text-purple-400" />
            <h2 className="text-lg font-semibold text-white">{t('settings.appearance')}</h2>
          </div>
          
          <div className="space-y-5">
            {/* 主题选择 */}
            <div>
              <div className="text-sm text-slate-400 mb-3">{t('settings.theme')}</div>
              <div className="grid grid-cols-3 gap-3">
                {Object.entries(themes).map(([key, themeConfig]) => (
                  <button
                    key={key}
                    onClick={() => setTheme(key)}
                    className={`p-4 rounded-xl border transition-all ${
                      theme === key 
                        ? 'border-indigo-500 bg-indigo-500/10' 
                        : 'border-white/5 hover:border-white/10 bg-white/5'
                    }`}
                  >
                    <div className="flex items-center justify-center mb-2">
                      {key === 'light' ? <Sun size={24} className="text-amber-400" /> : <Moon size={24} className="text-indigo-400" />}
                    </div>
                    <div className="text-sm text-white">{themeConfig.name}</div>
                  </button>
                ))}
              </div>
            </div>
            
            {/* 语言选择 */}
            <div className="pt-4 border-t border-white/5">
              <div className="text-sm text-slate-400 mb-3">{t('settings.language')}</div>
              <div className="grid grid-cols-3 gap-3">
                {Object.entries(locales).map(([key, name]) => (
                  <button
                    key={key}
                    onClick={() => setLocale(key)}
                    className={`p-3 rounded-xl border transition-all text-sm ${
                      locale === key 
                        ? 'border-indigo-500 bg-indigo-500/10 text-white' 
                        : 'border-white/5 hover:border-white/10 bg-white/5 text-slate-400'
                    }`}
                  >
                    {name}
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>

        {/* 操作按钮 */}
        <div className="flex items-center justify-end gap-3 animate-slide-up delay-300">
          <button
            onClick={handleReset}
            className="btn-ghost flex items-center gap-2"
          >
            <RotateCcw size={16} />
            <span>{t('settings.reset')}</span>
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            className="btn-aurora flex items-center gap-2"
          >
            {saving ? (
              <RefreshCw size={16} className="animate-spin" />
            ) : saved ? (
              <CheckCircle size={16} />
            ) : (
              <Save size={16} />
            )}
            <span>{saved ? t('settings.saved') : t('settings.save')}</span>
          </button>
        </div>
      </div>
    </div>
  )
}

export default Settings
