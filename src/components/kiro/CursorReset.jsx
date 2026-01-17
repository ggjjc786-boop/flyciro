import { useState, useRef, useEffect } from 'react'
import { 
  RefreshCw, Play, CheckCircle, XCircle, AlertCircle,
  Loader2, Shield, Fingerprint, HardDrive
} from 'lucide-react'
import { useDialog } from '../contexts/DialogContext'
import cursorApi from '../api/cursorApi'

function CursorReset() {
  const { showError, showSuccess, showConfirm } = useDialog()
  
  const [isRunning, setIsRunning] = useState(false)
  const [logs, setLogs] = useState([])
  const [currentMachineId, setCurrentMachineId] = useState(null)
  const [newMachineId, setNewMachineId] = useState(null)
  const [serviceOnline, setServiceOnline] = useState(false)

  const logsEndRef = useRef(null)

  useEffect(() => {
    logsEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [logs])

  useEffect(() => {
    checkServiceAndLoad()
  }, [])

  const checkServiceAndLoad = async () => {
    try {
      await cursorApi.healthCheck()
      setServiceOnline(true)
      loadCurrentMachineId()
    } catch (e) {
      setServiceOnline(false)
    }
  }

  const loadCurrentMachineId = async () => {
    try {
      const result = await cursorApi.getMachineId()
      if (result.success && result.data) {
        setCurrentMachineId(result.data.machineId || result.data.devDeviceId)
      }
    } catch (e) {
      console.error('Failed to get machine id:', e)
    }
  }

  const addLog = (message, type = 'info') => {
    const timestamp = new Date().toLocaleTimeString()
    setLogs(prev => [...prev, { timestamp, message, type }])
  }

  const clearLogs = () => {
    setLogs([])
    setNewMachineId(null)
  }

  const handleReset = async () => {
    const confirmed = await showConfirm(
      '确认重置',
      '重置机器码将清除 Cursor 的设备标识，这可能会影响已登录的账号。确定要继续吗？'
    )
    if (!confirmed) return

    clearLogs()
    setIsRunning(true)

    addLog('开始重置 Cursor 机器码...', 'info')

    try {
      addLog('正在检查配置文件...', 'info')
      const result = await cursorApi.resetMachineId()
      
      if (result.success) {
        addLog('机器码重置成功！', 'success')
        if (result.data) {
          setNewMachineId(result.data.machineId || result.data.devDeviceId)
          addLog(`新机器码: ${result.data.machineId || result.data.devDeviceId}`, 'success')
        }
        loadCurrentMachineId()
      } else {
        addLog(`重置失败: ${result.message}`, 'error')
      }
    } catch (e) {
      addLog(`错误: ${e.message}`, 'error')
    } finally {
      setIsRunning(false)
    }
  }

  const getLogIcon = (type) => {
    switch (type) {
      case 'success': return <CheckCircle size={14} className="text-emerald-400" />
      case 'error': return <XCircle size={14} className="text-red-400" />
      case 'warning': return <AlertCircle size={14} className="text-amber-400" />
      default: return <div className="w-2 h-2 rounded-full bg-cyan-400" />
    }
  }

  return (
    <div className="h-full p-6 overflow-auto">
      <div className="max-w-4xl mx-auto space-y-6">
        {/* 标题 */}
        <div className="flex items-center gap-3 animate-slide-up">
          <div className="w-12 h-12 rounded-2xl bg-gradient-to-br from-amber-500 to-orange-600 flex items-center justify-center shadow-lg">
            <RefreshCw size={24} className="text-white" />
          </div>
          <div>
            <h1 className="text-2xl font-bold text-white">重置机器码</h1>
            <p className="text-slate-500 text-sm">重置 Cursor 设备标识</p>
          </div>
        </div>

        {/* 警告提示 */}
        <div className="glass-card p-5 border-l-4 border-amber-500 animate-slide-up delay-100">
          <div className="flex items-start gap-4">
            <div className="w-10 h-10 rounded-xl bg-amber-500/20 flex items-center justify-center flex-shrink-0">
              <AlertCircle size={20} className="text-amber-400" />
            </div>
            <div>
              <h3 className="font-medium text-white mb-1">注意事项</h3>
              <ul className="text-sm text-slate-400 space-y-1">
                <li>• 重置机器码会清除 Cursor 的设备标识</li>
                <li>• 重置后需要重新登录 Cursor 账号</li>
                <li>• 频繁重置可能触发 Cursor 的安全检测</li>
                <li>• 建议在账号被限制时使用此功能</li>
              </ul>
            </div>
          </div>
        </div>

        <div className="grid grid-cols-2 gap-6">
          {/* 左侧：机器码信息 */}
          <div className="space-y-4">
            {/* 当前机器码 */}
            <div className="glass-card p-5 animate-slide-up delay-200">
              <h2 className="text-white font-medium mb-4 flex items-center gap-2">
                <Fingerprint size={18} className="text-cyan-400" />
                当前机器码
              </h2>
              <div className="p-4 rounded-xl bg-black/40">
                {currentMachineId ? (
                  <code className="text-cyan-400 font-mono text-sm break-all">
                    {currentMachineId}
                  </code>
                ) : (
                  <span className="text-slate-500">未检测到机器码</span>
                )}
              </div>
            </div>

            {/* 新机器码 */}
            {newMachineId && (
              <div className="glass-card p-5 border border-emerald-500/30 animate-scale-in">
                <h2 className="text-white font-medium mb-4 flex items-center gap-2">
                  <CheckCircle size={18} className="text-emerald-400" />
                  新机器码
                </h2>
                <div className="p-4 rounded-xl bg-black/40">
                  <code className="text-emerald-400 font-mono text-sm break-all">
                    {newMachineId}
                  </code>
                </div>
              </div>
            )}

            {/* 操作按钮 */}
            <button
              onClick={handleReset}
              disabled={isRunning}
              className="w-full btn-amber flex items-center justify-center gap-2"
            >
              {isRunning ? (
                <>
                  <Loader2 size={18} className="animate-spin" />
                  <span>重置中...</span>
                </>
              ) : (
                <>
                  <RefreshCw size={18} />
                  <span>重置机器码</span>
                </>
              )}
            </button>
          </div>

          {/* 右侧：日志 */}
          <div className="glass-card p-5 flex flex-col animate-slide-up delay-300">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-white font-medium">运行日志</h2>
              {isRunning && (
                <Loader2 size={14} className="animate-spin text-amber-400" />
              )}
            </div>
            <div className="flex-1 min-h-[300px] max-h-[400px] overflow-auto rounded-xl bg-black/40 p-4 font-mono text-sm">
              {logs.length === 0 ? (
                <div className="text-slate-600 text-center py-8">等待操作...</div>
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

        {/* 说明 */}
        <div className="glass-card p-5 animate-slide-up delay-400">
          <h2 className="text-white font-medium mb-3 flex items-center gap-2">
            <HardDrive size={18} className="text-slate-400" />
            工作原理
          </h2>
          <p className="text-sm text-slate-400">
            Cursor 使用机器码来标识设备，当账号在某设备上被限制时，可以通过重置机器码来绕过限制。
            此功能会修改 Cursor 存储的设备标识文件，生成新的随机机器码。
          </p>
        </div>
      </div>
    </div>
  )
}

export default CursorReset
