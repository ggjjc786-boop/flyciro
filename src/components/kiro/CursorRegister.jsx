import { useState, useEffect, useRef } from 'react'
import { 
  UserPlus, Play, Square, RefreshCw, CheckCircle, 
  XCircle, AlertCircle, Loader2, Settings, Mail
} from 'lucide-react'
import { useDialog } from '../contexts/DialogContext'
import cursorApi from '../api/cursorApi'

function CursorRegister() {
  const { showError, showSuccess } = useDialog()
  
  const [config, setConfig] = useState({
    emailDomain: '',
    emailUsername: '',
  })
  const [isRunning, setIsRunning] = useState(false)
  const [logs, setLogs] = useState([])
  const [currentStep, setCurrentStep] = useState('')
  const [showConfig, setShowConfig] = useState(false)
  const [registeredAccount, setRegisteredAccount] = useState(null)
  const [serviceOnline, setServiceOnline] = useState(false)
  const [pendingVerification, setPendingVerification] = useState(null)
  const [verificationCode, setVerificationCode] = useState('')

  const logsEndRef = useRef(null)
  const pollIntervalRef = useRef(null)

  useEffect(() => {
    logsEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [logs])

  useEffect(() => {
    checkServiceAndLoadConfig()
    return () => {
      if (pollIntervalRef.current) clearInterval(pollIntervalRef.current)
    }
  }, [])

  const checkServiceAndLoadConfig = async () => {
    try {
      await cursorApi.healthCheck()
      setServiceOnline(true)
      loadConfig()
    } catch (e) {
      setServiceOnline(false)
    }
  }

  const loadConfig = async () => {
    try {
      const result = await cursorApi.getConfig()
      if (result.success && result.data) {
        setConfig(result.data)
      }
    } catch (e) {
      console.error('Failed to load config:', e)
    }
  }

  const saveConfig = async () => {
    try {
      await cursorApi.updateConfig(config)
      await showSuccess('成功', '配置已保存')
      setShowConfig(false)
    } catch (e) {
      await showError('错误', `保存失败: ${e.message}`)
    }
  }

  const addLog = (message, type = 'info') => {
    const timestamp = new Date().toLocaleTimeString()
    setLogs(prev => [...prev, { timestamp, message, type }])
  }

  const clearLogs = () => {
    setLogs([])
    setCurrentStep('')
    setRegisteredAccount(null)
    setPendingVerification(null)
    setVerificationCode('')
  }

  const pollPendingVerifications = async () => {
    try {
      const result = await cursorApi.getPendingVerifications()
      if (result.success && result.data && result.data.length > 0) {
        setPendingVerification(result.data[0])
        return true
      }
      setPendingVerification(null)
      return false
    } catch (e) {
      return false
    }
  }

  const handleSubmitVerificationCode = async () => {
    if (!pendingVerification || !verificationCode || verificationCode.length !== 6) {
      await showError('错误', '请输入6位验证码')
      return
    }
    
    try {
      await cursorApi.submitVerificationCode(pendingVerification.email_id, verificationCode)
      addLog(`已提交验证码: ${verificationCode}`, 'success')
      setPendingVerification(null)
      setVerificationCode('')
    } catch (e) {
      await showError('错误', `提交失败: ${e.message}`)
    }
  }

  const handleStartRegister = async () => {
    if (!serviceOnline) {
      await showError('错误', 'Cursor 服务未启动，请先启动后端服务')
      return
    }

    clearLogs()
    setIsRunning(true)

    addLog('开始监听注册流程...', 'info')
    addLog('请在 cursor-service 目录运行 Python 后端进行注册', 'info')
    addLog('命令: python start.py', 'info')
    addLog('', 'info')
    addLog('等待验证码请求...', 'info')
    
    pollIntervalRef.current = setInterval(async () => {
      const hasPending = await pollPendingVerifications()
      if (hasPending) {
        addLog('检测到验证码请求，请查看邮箱并输入验证码', 'warning')
      }
    }, 3000)
  }

  const handleStopRegister = () => {
    if (pollIntervalRef.current) {
      clearInterval(pollIntervalRef.current)
      pollIntervalRef.current = null
    }
    setIsRunning(false)
    addLog('已停止监听', 'warning')
  }

  const getLogIcon = (type) => {
    switch (type) {
      case 'success': return <CheckCircle size={14} className="text-emerald-400" />
      case 'error': return <XCircle size={14} className="text-red-400" />
      case 'warning': return <AlertCircle size={14} className="text-amber-400" />
      default: return <div className="w-2 h-2 rounded-full bg-cyan-400" />
    }
  }

  if (!serviceOnline) {
    return (
      <div className="h-full p-6 overflow-auto">
        <div className="max-w-4xl mx-auto">
          <div className="glass-card p-8 text-center">
            <AlertCircle size={48} className="text-amber-400 mx-auto mb-4" />
            <h2 className="text-xl font-bold text-white mb-2">Cursor 服务未启动</h2>
            <p className="text-slate-400 mb-4">请先启动 cursor-service 后端服务</p>
            <code className="block bg-black/40 p-4 rounded-lg text-cyan-400 text-sm mb-4 text-left">
              cd cursor-service<br/>
              pip install -r requirements.txt<br/>
              python start.py
            </code>
            <button onClick={checkServiceAndLoadConfig} className="btn-cyan">
              <RefreshCw size={16} className="mr-2" />
              重新检测
            </button>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="h-full p-6 overflow-auto">
      <div className="max-w-5xl mx-auto space-y-6">
        {/* 标题 */}
        <div className="flex items-center justify-between animate-slide-up">
          <div className="flex items-center gap-3">
            <div className="w-12 h-12 rounded-2xl bg-gradient-to-br from-cyan-500 to-blue-600 flex items-center justify-center shadow-lg">
              <UserPlus size={24} className="text-white" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-white">Cursor 自动注册</h1>
              <p className="text-slate-500 text-sm">自动创建 Cursor 账号</p>
            </div>
          </div>
          <button
            onClick={() => setShowConfig(!showConfig)}
            className={`btn-ghost flex items-center gap-2 ${showConfig ? 'bg-white/10' : ''}`}
          >
            <Settings size={18} />
            <span>配置</span>
          </button>
        </div>

        {/* 配置面板 */}
        {showConfig && (
          <div className="glass-card p-5 animate-scale-in border-cyan-500/20">
            <h2 className="text-white font-medium mb-4 flex items-center gap-2">
              <Settings size={18} className="text-cyan-400" />
              邮箱配置
            </h2>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="text-sm text-slate-400 mb-2 block">邮箱域名</label>
                <input
                  type="text"
                  value={config.emailDomain}
                  onChange={(e) => setConfig(prev => ({ ...prev, emailDomain: e.target.value }))}
                  placeholder="example.com"
                  className="input-aurora"
                />
                <p className="text-xs text-slate-600 mt-1">已配置 Cloudflare 邮件路由的域名</p>
              </div>
              <div>
                <label className="text-sm text-slate-400 mb-2 block">邮箱用户名</label>
                <input
                  type="text"
                  value={config.emailUsername}
                  onChange={(e) => setConfig(prev => ({ ...prev, emailUsername: e.target.value }))}
                  placeholder="ddcat"
                  className="input-aurora"
                />
                <p className="text-xs text-slate-600 mt-1">tempmail.plus 获取的邮箱前缀</p>
              </div>
            </div>
            <div className="flex justify-end mt-4">
              <button onClick={saveConfig} className="btn-aurora">
                <span>保存配置</span>
              </button>
            </div>
          </div>
        )}

        <div className="grid grid-cols-2 gap-6">
          {/* 左侧：控制区域 */}
          <div className="space-y-4">
            {/* 注册信息预览 */}
            <div className="glass-card p-5 animate-slide-up delay-100">
              <h2 className="text-white font-medium mb-4">注册配置</h2>
              <div className="space-y-3 text-sm">
                <div className="flex items-center justify-between p-3 rounded-lg bg-black/30">
                  <span className="text-slate-500">邮箱域名</span>
                  <span className="text-cyan-400 font-mono">{config.emailDomain || '未配置'}</span>
                </div>
                <div className="flex items-center justify-between p-3 rounded-lg bg-black/30">
                  <span className="text-slate-500">邮箱前缀</span>
                  <span className="text-cyan-400 font-mono">{config.emailUsername || '未配置'}</span>
                </div>
                <div className="flex items-center justify-between p-3 rounded-lg bg-black/30">
                  <span className="text-slate-500">服务状态</span>
                  <span className="text-emerald-400 flex items-center gap-1">
                    <span className="w-2 h-2 rounded-full bg-emerald-400"></span>
                    在线
                  </span>
                </div>
              </div>
            </div>

            {/* 验证码输入 */}
            {pendingVerification && (
              <div className="glass-card p-5 border border-amber-500/30 animate-scale-in">
                <h2 className="text-white font-medium mb-3 flex items-center gap-2">
                  <Mail size={18} className="text-amber-400" />
                  输入验证码
                </h2>
                <p className="text-sm text-slate-400 mb-3">
                  请查看邮箱 <span className="text-cyan-400">{pendingVerification.email}</span> 中的验证码
                </p>
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
                    onClick={handleSubmitVerificationCode}
                    disabled={verificationCode.length !== 6}
                    className="btn-aurora"
                  >
                    提交
                  </button>
                </div>
              </div>
            )}

            {/* 注册成功信息 */}
            {registeredAccount && (
              <div className="glass-card p-5 border border-emerald-500/30 animate-scale-in">
                <h2 className="text-white font-medium mb-3 flex items-center gap-2">
                  <CheckCircle size={18} className="text-emerald-400" />
                  注册成功
                </h2>
                <div className="space-y-2 text-sm">
                  <div className="flex items-center justify-between p-2 rounded bg-black/30">
                    <span className="text-slate-500">邮箱</span>
                    <code className="text-emerald-400">{registeredAccount.email}</code>
                  </div>
                  <div className="flex items-center justify-between p-2 rounded bg-black/30">
                    <span className="text-slate-500">密码</span>
                    <code className="text-emerald-400">{registeredAccount.password}</code>
                  </div>
                </div>
              </div>
            )}

            {/* 操作按钮 */}
            <div className="flex gap-3">
              {!isRunning ? (
                <button
                  onClick={handleStartRegister}
                  className="btn-cyan flex-1 flex items-center justify-center gap-2"
                >
                  <Play size={18} />
                  <span>开始监听</span>
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
          <div className="glass-card p-5 flex flex-col animate-slide-up delay-200">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-white font-medium">运行日志</h2>
              {isRunning && (
                <div className="flex items-center gap-2">
                  <Loader2 size={14} className="animate-spin text-cyan-400" />
                  <span className="text-xs text-slate-500">{currentStep || '监听中'}</span>
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

export default CursorRegister
