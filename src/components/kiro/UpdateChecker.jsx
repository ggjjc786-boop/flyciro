import { useState, useEffect } from 'react'
import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'
import { Download, X, RefreshCw, AlertTriangle, Loader2 } from 'lucide-react'

const API_BASE = 'https://pe.xphdfs.me'

export default function UpdateChecker() {
  const [updateInfo, setUpdateInfo] = useState(null)
  const [showModal, setShowModal] = useState(false)
  const [downloading, setDownloading] = useState(false)
  const [progress, setProgress] = useState(0)
  const [status, setStatus] = useState('')
  const [error, setError] = useState('')

  useEffect(() => {
    checkForUpdate()
  }, [])

  const checkForUpdate = async () => {
    try {
      const currentVersion = await getVersion()
      const res = await fetch(`${API_BASE}/api/check-update?current_version=${currentVersion}`)
      const data = await res.json()
      
      if (data.success && data.has_update) {
        setUpdateInfo(data.update)
        setShowModal(true)
      }
    } catch (e) {
      console.error('检查更新失败:', e)
    }
  }

  const formatFileSize = (bytes) => {
    if (!bytes) return '未知'
    if (bytes < 1024) return bytes + ' B'
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  }

  const handleDownload = async () => {
    if (!updateInfo?.download_url) return
    
    setDownloading(true)
    setError('')
    setProgress(0)
    setStatus('准备下载...')
    
    try {
      // 构建完整下载 URL
      const downloadUrl = updateInfo.download_url.startsWith('http') 
        ? updateInfo.download_url 
        : `${API_BASE}${updateInfo.download_url}`
      
      setStatus('下载中...')
      
      // 使用 fetch 下载
      const response = await fetch(downloadUrl)
      if (!response.ok) throw new Error('下载失败: ' + response.status)
      
      const contentLength = response.headers.get('content-length')
      const total = parseInt(contentLength, 10) || updateInfo.file_size || 0
      let loaded = 0
      
      const reader = response.body.getReader()
      const chunks = []
      
      while (true) {
        const { done, value } = await reader.read()
        if (done) break
        chunks.push(value)
        loaded += value.length
        if (total > 0) {
          setProgress(Math.round((loaded / total) * 100))
        }
      }
      
      // 合并数据为 Uint8Array
      const totalLength = chunks.reduce((acc, chunk) => acc + chunk.length, 0)
      const fileData = new Uint8Array(totalLength)
      let offset = 0
      for (const chunk of chunks) {
        fileData.set(chunk, offset)
        offset += chunk.length
      }
      
      setProgress(100)
      setStatus('下载完成，正在启动安装程序...')
      
      // 调用 Rust 后端保存文件并运行
      const fileName = `KiroRefillTool-${updateInfo.version}.exe`
      await invoke('save_and_run_update', { 
        fileName, 
        fileData: Array.from(fileData) 
      })
      
    } catch (e) {
      console.error('更新失败:', e)
      
      // 如果 Rust 命令不存在，回退到浏览器下载
      if (e.toString().includes('save_and_run_update')) {
        setStatus('正在打开下载链接...')
        const downloadUrl = updateInfo.download_url.startsWith('http') 
          ? updateInfo.download_url 
          : `${API_BASE}${updateInfo.download_url}`
        window.open(downloadUrl, '_blank')
        setError('请手动运行下载的安装程序')
      } else {
        setError('更新失败: ' + e.message)
      }
      setStatus('')
    } finally {
      setDownloading(false)
    }
  }

  if (!showModal || !updateInfo) return null

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/60 backdrop-blur-sm animate-fade-in">
      <div className="w-full max-w-md glass-card p-6 m-4 animate-scale-in">
        {/* 头部 */}
        <div className="flex items-start justify-between mb-6">
          <div className="flex items-center gap-3">
            <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-emerald-500 to-green-600 flex items-center justify-center">
              <RefreshCw size={24} className="text-white" />
            </div>
            <div>
              <h2 className="text-xl font-bold text-white">发现新版本</h2>
              <p className="text-slate-400 text-sm">v{updateInfo.version}</p>
            </div>
          </div>
          {!updateInfo.is_force && !downloading && (
            <button
              onClick={() => setShowModal(false)}
              className="p-2 rounded-lg hover:bg-white/10 transition-colors"
            >
              <X size={20} className="text-slate-400" />
            </button>
          )}
        </div>

        {/* 更新内容 */}
        {updateInfo.changelog && (
          <div className="mb-6">
            <h3 className="text-sm font-medium text-slate-300 mb-2">更新内容</h3>
            <div className="p-4 rounded-xl bg-white/5 max-h-40 overflow-y-auto">
              <pre className="text-sm text-slate-400 whitespace-pre-wrap font-sans">
                {updateInfo.changelog}
              </pre>
            </div>
          </div>
        )}

        {/* 文件大小 */}
        <div className="flex items-center justify-between text-sm mb-6">
          <span className="text-slate-500">文件大小</span>
          <span className="text-slate-300">{formatFileSize(updateInfo.file_size)}</span>
        </div>

        {/* 强制更新提示 */}
        {updateInfo.is_force && (
          <div className="mb-4 p-3 rounded-xl bg-amber-500/10 border border-amber-500/20 flex items-center gap-2">
            <AlertTriangle size={18} className="text-amber-400" />
            <span className="text-sm text-amber-400">此版本为强制更新，请立即更新</span>
          </div>
        )}

        {/* 错误提示 */}
        {error && (
          <div className="mb-4 p-3 rounded-xl bg-red-500/10 border border-red-500/20 text-sm text-red-400">
            {error}
          </div>
        )}

        {/* 下载进度 */}
        {(downloading || status) && (
          <div className="mb-4">
            <div className="flex items-center justify-between text-sm mb-2">
              <span className="text-slate-400">{status || '处理中...'}</span>
              <span className="text-slate-300">{progress}%</span>
            </div>
            <div className="h-2 rounded-full bg-white/10 overflow-hidden">
              <div 
                className="h-full bg-gradient-to-r from-emerald-500 to-green-500 transition-all duration-300"
                style={{ width: `${progress}%` }}
              />
            </div>
          </div>
        )}

        {/* 按钮 */}
        <div className="flex gap-3">
          {!updateInfo.is_force && !downloading && (
            <button
              onClick={() => setShowModal(false)}
              className="flex-1 btn-ghost"
            >
              稍后再说
            </button>
          )}
          <button
            onClick={handleDownload}
            disabled={downloading}
            className="flex-1 btn-aurora flex items-center justify-center gap-2"
          >
            {downloading ? (
              <>
                <Loader2 size={18} className="animate-spin" />
                <span>{progress < 100 ? '下载中' : '安装中'}</span>
              </>
            ) : (
              <>
                <Download size={18} />
                <span>立即更新</span>
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  )
}
