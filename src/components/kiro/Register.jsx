import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { 
  UserPlus, Mail, Play, Square, RefreshCw, CheckCircle, 
  XCircle, AlertCircle, ChevronDown, Loader2, Crown, Lock
} from 'lucide-react'
import { useDialog } from '../contexts/DialogContext'
import { useI18n } from '../i18n.jsx'

function Register({ userInfo }) {
  const { showError, showSuccess } = useDialog()
  const { t } = useI18n()
  
  const isVip = userInfo?.is_vip || false

  const [email, setEmail] = useState('')
  const [isRunning, setIsRunning] = useState(false)
  const [logs, setLogs] = useState([])
  const [currentStep, setCurrentStep] = useState('')
  const [verificationCode, setVerificationCode] = useState('')
  const [waitingForCode, setWaitingForCode] = useState(false)
  const [generatedPassword, setGeneratedPassword] = useState('')
  const [selectedBrowser, setSelectedBrowser] = useState(null)
  const [detectedBrowsers, setDetectedBrowsers] = useState([])
  const [showBrowserList, setShowBrowserList] = useState(false)

  const logsEndRef = useRef(null)
  const unlistenRef = useRef(null)

  useEffect(() => {
    logsEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [logs])

  useEffect(() => {
    detectBrowsers()
    return () => { if (unlistenRef.current) unlistenRef.current() }
  }, [])

  const detectBrowsers = async () => {
    try {
      const browsers = await invoke('detect_installed_browsers')
      setDetectedBrowsers(browsers)
      const defaultBrowser = browsers.find(b => b.incognitoArg) || browsers[0]
      if (defaultBrowser && !selectedBrowser) setSelectedBrowser(defaultBrowser)
    } catch (e) {
      console.error('Failed to detect browsers:', e)
    }
  }

  const addLog = (message, type = 'info') => {
    const timestamp = new Date().toLocaleTimeString()
    setLogs(prev => [...prev, { timestamp, message, type }])
  }

  const clearLogs = () => {
    setLogs([])
    setCurrentStep('')
    setGeneratedPassword('')
  }

  const generatePassword = () => {
    const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789'
    const special = '!@#$%^&*'
    let password = ''
    for (let i = 0; i < 12; i++) {
      password += chars.charAt(Math.floor(Math.random() * chars.length))
    }
    password += special.charAt(Math.floor(Math.random() * special.length))
    return password
  }

  const generateRandomName = () => {
    const firstNames = ['James', 'John', 'Robert', 'Michael', 'William', 'David', 'Mary', 'Patricia', 'Jennifer', 'Linda']
    const lastNames = ['Smith', 'Johnson', 'Williams', 'Brown', 'Jones', 'Garcia', 'Miller', 'Davis', 'Wilson', 'Anderson']
    return `${firstNames[Math.floor(Math.random() * firstNames.length)]} ${lastNames[Math.floor(Math.random() * lastNames.length)]}`
  }

  const handleStartRegister = async () => {
    if (!email) {
      await showError(t('register.error'), t('register.emailRequired'))
      return
    }
    if (!selectedBrowser) {
      await showError(t('register.error'), t('register.browserRequired'))
      return
    }
    if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
      await showError(t('register.error'), t('register.invalidEmail'))
      return
    }

    clearLogs()
    setIsRunning(true)
    setWaitingForCode(false)
    setVerificationCode('')

    const password = generatePassword()
    setGeneratedPassword(password)
    const randomName = generateRandomName()

    addLog(`开始注册流程...`, 'info')
    addLog(`邮箱: ${email}`, 'info')
    addLog(`生成密码: ${password}`, 'success')

    try {
      unlistenRef.current = await listen('register-progress', (event) => {
        const { step, message, type } = event.payload
        setCurrentStep(step)
        addLog(message, type || 'info')
        if (step === 'waiting_for_code') setWaitingForCode(true)
        else if (step === 'completed' || step === 'error') {
          setIsRunning(false)
          setWaitingForCode(false)
        }
      })

      await invoke('start_registration', {
        email,
        password,
        name: randomName,
        browserPath: selectedBrowser.path,
        incognitoArg: selectedBrowser.incognitoArg
      })
    } catch (e) {
      addLog(`错误: ${e}`, 'error')
      setIsRunning(false)
      setWaitingForCode(false)
    }
  }

  const handleSubmitCode = async () => {
    if (!verificationCode || verificationCode.length !== 6) {
      await showError(t('register.error'), t('register.invalidCode'))
      return
    }
    addLog(`提交验证码: ${verificationCode}`, 'info')
    try {
      await invoke('submit_verification_code', { code: verificationCode })
      setWaitingForCode(false)
    } catch (e) {
      addLog(`错误: ${e}`, 'error')
    }
  }

  const handleStopRegister = async () => {
    try {
      await invoke('stop_registration')
      addLog('已停止注册', 'warning')
      setIsRunning(false)
      setWaitingForCode(false)
    } catch (e) {}
  }

  const getLogIcon = (type) => {
    switch (type) {
      case 'success': return <CheckCircle size={14} className="text-emerald-400" />
      case 'error': return <XCircle size={14} className="text-red-400" />
      case 'warning': return <AlertCircle size={14} className="text-amber-400" />
      default: return <div className="w-2 h-2 rounded-full bg-indigo-400" />
    }
  }

  return (
    <div className="h-full p-6 overflow-auto">
      <div className="max-w-5xl mx-auto space-y-6">
        {/* 标题 */}
        <div className="flex items-center gap-3 animate-slide-up">
          <div className="w-12 h-12 rounded-2xl bg-gradient-to-br from-emerald-500 to-green-600 flex items-center justify-center shadow-lg">
            <UserPlus size={24} className="text-white" />
          </div>
          <div>
            <h1 className="text-2xl font-bold text-white">{t('register.title')}</h1>
            <p className="text-slate-500 text-sm">{t('register.subtitle')}</p>
          </div>
        </div>

        {/* VIP 限制提示 */}
        {!isVip && (
          <div className="glass-card p-6 border-l-4 border-amber-500 animate-slide-up delay-100">
            <div className="flex items-center gap-4">
              <div className="w-14 h-14 rounded-2xl bg-gradient-to-br from-amber-400 to-orange-500 flex items-center justify-center shadow-lg">
                <Lock size={28} className="text-white" />
              </div>
              <div className="flex-1">
                <h3 className="text-lg font-semibold text-white flex items-center gap-2">
                  <Crown size={20} className="text-amber-400" />
                  会员专属功能
                </h3>
                <p className="text-slate-400 mt-1 text-sm">
                  注册账号功能仅限会员使用，请先开通会员后再使用此功能。
                </p>
              </div>
              <a 
                href="https://pe.xphdfs.me" 
                target="_blank" 
                rel="noopener noreferrer"
                className="btn-aurora"
              >
                <span>开通会员</span>
              </a>
            </div>
          </div>
        )}

        <div className={`grid grid-cols-2 gap-6 ${!isVip ? 'opacity-40 pointer-events-none' : ''}`}>
          {/* 左侧：输入区域 */}
          <div className="space-y-4">
            {/* 浏览器选择 */}
            <div className="glass-card p-5 animate-slide-up delay-100">
              <h2 className="text-white font-medium mb-4">选择浏览器</h2>
              <div className="relative">
                <button
                  onClick={() => setShowBrowserList(!showBrowserList)}
                  disabled={isRunning}
                  className="w-full input-aurora flex items-center justify-between"
                >
                  <span className="text-slate-300">
                    {selectedBrowser ? (
                      <span className="flex items-center gap-2">
                        {selectedBrowser.name}
                        {selectedBrowser.incognitoArg && (
                          <span className="badge badge-aurora text-xs">无痕</span>
                        )}
                      </span>
                    ) : '选择浏览器'}
                  </span>
                  <ChevronDown size={16} className={`text-slate-500 transition-transform ${showBrowserList ? 'rotate-180' : ''}`} />
                </button>
                {showBrowserList && (
                  <div className="absolute z-10 w-full mt-2 glass-card rounded-xl overflow-hidden">
                    {detectedBrowsers.map((browser, index) => (
                      <button
                        key={index}
                        onClick={() => { setSelectedBrowser(browser); setShowBrowserList(false) }}
                        className="w-full px-4 py-3 text-left text-slate-300 hover:bg-white/5 flex items-center justify-between"
                      >
                        {browser.name}
                        {browser.incognitoArg && <span className="badge badge-aurora text-xs">无痕</span>}
                      </button>
                    ))}
                  </div>
                )}
              </div>
            </div>

            {/* 邮箱输入 */}
            <div className="glass-card p-5 animate-slide-up delay-200">
              <h2 className="text-white font-medium mb-4">输入邮箱</h2>
              <div className="relative">
                <Mail size={18} className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-500" />
                <input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="your@email.com"
                  disabled={isRunning}
                  className="input-aurora pl-12"
                />
              </div>
            </div>

            {/* 验证码输入 */}
            {waitingForCode && (
              <div className="glass-card p-5 border border-amber-500/30 animate-scale-in">
                <h2 className="text-white font-medium mb-2 flex items-center gap-2">
                  <AlertCircle size={18} className="text-amber-400" />
                  输入验证码
                </h2>
                <p className="text-sm text-slate-500 mb-4">请查收邮箱中的验证码</p>
                <div className="flex gap-3">
                  <input
                    type="text"
                    value={verificationCode}
                    onChange={(e) => setVerificationCode(e.target.value.replace(/\D/g, '').slice(0, 6))}
                    placeholder="000000"
                    maxLength={6}
                    className="input-aurora flex-1 text-center text-2xl tracking-widest font-mono"
                  />
                  <button
                    onClick={handleSubmitCode}
                    disabled={verificationCode.length !== 6}
                    className="btn-aurora"
                  >
                    <span>提交</span>
                  </button>
                </div>
              </div>
            )}

            {/* 生成的密码 */}
            {generatedPassword && (
              <div className="glass-card p-5 animate-slide-up">
                <h2 className="text-white font-medium mb-3">生成的凭据</h2>
                <div className="p-4 rounded-xl bg-black/30">
                  <div className="flex items-center justify-between">
                    <span className="text-slate-500">密码:</span>
                    <code className="font-mono text-emerald-400">{generatedPassword}</code>
                  </div>
                  <p className="text-xs text-slate-600 mt-2">请妥善保存此密码</p>
                </div>
              </div>
            )}

            {/* 操作按钮 */}
            <div className="flex gap-3">
              {!isRunning ? (
                <button
                  onClick={handleStartRegister}
                  disabled={!email || !selectedBrowser}
                  className="btn-aurora flex-1 flex items-center justify-center gap-2"
                >
                  <Play size={18} />
                  <span>开始注册</span>
                </button>
              ) : (
                <button
                  onClick={handleStopRegister}
                  className="flex-1 px-6 py-3 bg-red-500/20 text-red-400 border border-red-500/30 rounded-xl hover:bg-red-500/30 transition-colors flex items-center justify-center gap-2"
                >
                  <Square size={18} />
                  <span>停止</span>
                </button>
              )}
              <button
                onClick={clearLogs}
                disabled={isRunning}
                className="btn-ghost"
              >
                <RefreshCw size={18} />
              </button>
            </div>
          </div>

          {/* 右侧：日志 */}
          <div className="glass-card p-5 flex flex-col animate-slide-up delay-300">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-white font-medium">运行日志</h2>
              {isRunning && (
                <div className="flex items-center gap-2">
                  <Loader2 size={14} className="animate-spin text-indigo-400" />
                  <span className="text-xs text-slate-500">{currentStep}</span>
                </div>
              )}
            </div>
            <div className="flex-1 min-h-[400px] max-h-[500px] overflow-auto rounded-xl bg-black/40 p-4 font-mono text-sm">
              {logs.length === 0 ? (
                <div className="text-slate-600 text-center py-8">等待开始...</div>
              ) : (
                <div className="space-y-2">
                  {logs.map((log, index) => (
                    <div key={index} className="flex items-start gap-2">
                      <span className="text-slate-600 flex-shrink-0">[{log.timestamp}]</span>
                      <span className="flex-shrink-0 mt-0.5">{getLogIcon(log.type)}</span>
                      <span className={
                        log.type === 'error' ? 'text-red-400' :
                        log.type === 'success' ? 'text-emerald-400' :
                        log.type === 'warning' ? 'text-amber-400' :
                        'text-slate-400'
                      }>{log.message}</span>
                    </div>
                  ))}
                  <div ref={logsEndRef} />
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default Register
