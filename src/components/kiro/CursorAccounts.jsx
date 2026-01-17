import { useState, useEffect } from 'react'
import { 
  Key, Search, Plus, Trash2, RefreshCw, Copy, 
  CheckCircle, XCircle, Clock, MoreVertical,
  MousePointer2, Download, Upload, AlertCircle
} from 'lucide-react'
import { useDialog } from '../contexts/DialogContext'
import cursorApi from '../api/cursorApi'

function CursorAccounts() {
  const { showError, showSuccess, showConfirm } = useDialog()
  
  const [accounts, setAccounts] = useState([])
  const [loading, setLoading] = useState(true)
  const [searchTerm, setSearchTerm] = useState('')
  const [selectedAccounts, setSelectedAccounts] = useState([])
  const [serviceOnline, setServiceOnline] = useState(false)

  useEffect(() => {
    checkServiceAndLoad()
  }, [])

  const checkServiceAndLoad = async () => {
    try {
      await cursorApi.healthCheck()
      setServiceOnline(true)
      loadAccounts()
    } catch (e) {
      setServiceOnline(false)
      setLoading(false)
    }
  }

  const loadAccounts = async () => {
    setLoading(true)
    try {
      const result = await cursorApi.getAccounts(1, 100, searchTerm)
      if (result.success) {
        setAccounts(result.data || [])
      }
    } catch (e) {
      console.error(e)
      setAccounts([])
    } finally {
      setLoading(false)
    }
  }

  const handleDelete = async (id) => {
    const confirmed = await showConfirm('确认删除', '确定要删除这个账号吗？')
    if (!confirmed) return
    
    try {
      await cursorApi.deleteAccount(id)
      await showSuccess('成功', '账号已删除')
      loadAccounts()
    } catch (e) {
      await showError('错误', `删除失败: ${e.message}`)
    }
  }

  const handleCopy = async (text, label) => {
    try {
      await navigator.clipboard.writeText(text)
      await showSuccess('已复制', `${label}已复制到剪贴板`)
    } catch (e) {
      await showError('错误', '复制失败')
    }
  }

  const handleExport = async () => {
    try {
      const data = JSON.stringify(accounts, null, 2)
      const blob = new Blob([data], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `cursor-accounts-${new Date().toISOString().split('T')[0]}.json`
      a.click()
      URL.revokeObjectURL(url)
      await showSuccess('成功', '账号已导出')
    } catch (e) {
      await showError('错误', `导出失败: ${e}`)
    }
  }

  const getStatusBadge = (status) => {
    switch (status) {
      case '正常':
      case 'active':
        return <span className="badge badge-success">正常</span>
      case '已封禁':
      case 'banned':
        return <span className="badge badge-error">已封禁</span>
      case '过期':
      case 'expired':
        return <span className="badge badge-warning">已过期</span>
      default:
        return <span className="badge badge-default">{status || '未知'}</span>
    }
  }

  const filteredAccounts = accounts.filter(acc => 
    acc.email?.toLowerCase().includes(searchTerm.toLowerCase())
  )

  return (
    <div className="h-full p-6 overflow-auto">
      <div className="max-w-6xl mx-auto space-y-6">
        {/* 标题 */}
        <div className="flex items-center justify-between animate-slide-up">
          <div className="flex items-center gap-3">
            <div className="w-12 h-12 rounded-2xl bg-gradient-to-br from-cyan-500 to-blue-600 flex items-center justify-center shadow-lg">
              <Key size={24} className="text-white" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-white">Cursor 账号管理</h1>
              <p className="text-slate-500 text-sm">管理已注册的 Cursor 账号</p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <button onClick={handleExport} className="btn-ghost">
              <Download size={18} />
              <span>导出</span>
            </button>
            <button onClick={loadAccounts} className="btn-ghost">
              <RefreshCw size={18} className={loading ? 'animate-spin' : ''} />
            </button>
          </div>
        </div>

        {/* 搜索栏 */}
        <div className="glass-card p-4 animate-slide-up delay-100">
          <div className="relative">
            <Search size={18} className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-500" />
            <input
              type="text"
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              placeholder="搜索邮箱..."
              className="input-aurora pl-12"
            />
          </div>
        </div>

        {/* 账号列表 */}
        <div className="glass-card overflow-hidden animate-slide-up delay-200">
          {!serviceOnline ? (
            <div className="p-8 text-center">
              <AlertCircle size={48} className="text-amber-400 mx-auto mb-4" />
              <p className="text-white font-medium mb-2">Cursor 服务未启动</p>
              <p className="text-slate-500 text-sm mb-4">请先启动 cursor-service 后端服务</p>
              <code className="block bg-black/40 p-3 rounded-lg text-cyan-400 text-sm mb-4">
                cd cursor-service && python start.py
              </code>
              <button onClick={checkServiceAndLoad} className="btn-cyan">
                <RefreshCw size={16} className="mr-2" />
                重新检测
              </button>
            </div>
          ) : loading ? (
            <div className="p-8 text-center">
              <RefreshCw size={24} className="animate-spin text-cyan-400 mx-auto mb-2" />
              <p className="text-slate-500">加载中...</p>
            </div>
          ) : filteredAccounts.length === 0 ? (
            <div className="p-8 text-center">
              <MousePointer2 size={48} className="text-slate-600 mx-auto mb-4" />
              <p className="text-slate-500">暂无 Cursor 账号</p>
              <p className="text-slate-600 text-sm mt-1">使用自动注册功能创建新账号</p>
            </div>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-white/5">
                    <th className="text-left p-4 text-slate-500 font-medium text-sm">邮箱</th>
                    <th className="text-left p-4 text-slate-500 font-medium text-sm">密码</th>
                    <th className="text-left p-4 text-slate-500 font-medium text-sm">状态</th>
                    <th className="text-left p-4 text-slate-500 font-medium text-sm">创建时间</th>
                    <th className="text-right p-4 text-slate-500 font-medium text-sm">操作</th>
                  </tr>
                </thead>
                <tbody>
                  {filteredAccounts.map((account) => (
                    <tr key={account.id} className="border-b border-white/5 hover:bg-white/5 transition-colors">
                      <td className="p-4">
                        <div className="flex items-center gap-2">
                          <span className="text-white font-mono text-sm">{account.email}</span>
                          <button
                            onClick={() => handleCopy(account.email, '邮箱')}
                            className="p-1 rounded hover:bg-white/10 transition-colors"
                          >
                            <Copy size={14} className="text-slate-500" />
                          </button>
                        </div>
                      </td>
                      <td className="p-4">
                        <div className="flex items-center gap-2">
                          <span className="text-slate-400 font-mono text-sm">
                            {account.password ? '••••••••' : '-'}
                          </span>
                          {account.password && (
                            <button
                              onClick={() => handleCopy(account.password, '密码')}
                              className="p-1 rounded hover:bg-white/10 transition-colors"
                            >
                              <Copy size={14} className="text-slate-500" />
                            </button>
                          )}
                        </div>
                      </td>
                      <td className="p-4">
                        {getStatusBadge(account.status)}
                      </td>
                      <td className="p-4">
                        <span className="text-slate-500 text-sm">
                          {account.createdAt ? new Date(account.createdAt).toLocaleDateString() : '-'}
                        </span>
                      </td>
                      <td className="p-4 text-right">
                        <button
                          onClick={() => handleDelete(account.id)}
                          className="p-2 rounded-lg hover:bg-red-500/20 transition-colors group"
                        >
                          <Trash2 size={16} className="text-slate-500 group-hover:text-red-400" />
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>

        {/* 统计信息 */}
        <div className="flex items-center justify-between text-sm text-slate-500 animate-slide-up delay-300">
          <span>共 {filteredAccounts.length} 个账号</span>
          <span>
            正常: {filteredAccounts.filter(a => a.status === '正常' || a.status === 'active').length} | 
            封禁: {filteredAccounts.filter(a => a.status === '已封禁' || a.status === 'banned').length}
          </span>
        </div>
      </div>
    </div>
  )
}

export default CursorAccounts
