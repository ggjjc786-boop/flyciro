import { useState, useEffect } from 'react'
import { Mail, Lock, User, Shield, ArrowRight, Loader2, CheckCircle, Crown, ArrowLeft, Phone, KeyRound } from 'lucide-react'

const API_BASE = 'https://pe.xphdfs.me'

export default function UserAuth({ onVerified, onBack }) {
  const [mode, setMode] = useState('login') // login, register
  const [loginMethod, setLoginMethod] = useState('code') // code, password
  const [accountType, setAccountType] = useState('email') // email, phone
  const [email, setEmail] = useState('')
  const [phone, setPhone] = useState('')
  const [password, setPassword] = useState('')
  const [nickname, setNickname] = useState('')
  const [code, setCode] = useState('')
  const [loading, setLoading] = useState(false)
  const [sendingCode, setSendingCode] = useState(false)
  const [countdown, setCountdown] = useState(0)
  const [toast, setToast] = useState({ show: false, message: '', type: 'success' })

  useEffect(() => {
    const savedUser = localStorage.getItem('kiro_user')
    if (savedUser) {
      try {
        const userData = JSON.parse(savedUser)
        onVerified({
          code: userData.email || userData.phone,
          nickname: userData.nickname,
          remaining_quota: userData.is_vip ? -1 : 0,
          total_quota: userData.is_vip ? -1 : 0,
          expires_at: userData.vip_expires_at || null,
          is_vip: userData.is_vip || false
        })
      } catch (e) {
        localStorage.removeItem('kiro_user')
      }
    }
  }, [])

  useEffect(() => {
    if (countdown > 0) {
      const timer = setTimeout(() => setCountdown(countdown - 1), 1000)
      return () => clearTimeout(timer)
    }
  }, [countdown])

  // Toast 自动消失
  useEffect(() => {
    if (toast.show) {
      const timer = setTimeout(() => setToast({ ...toast, show: false }), 3000)
      return () => clearTimeout(timer)
    }
  }, [toast.show])

  const showToast = (message, type = 'success') => {
    setToast({ show: true, message, type })
  }

  const handleSendCode = async () => {
    const identifier = accountType === 'email' ? email : phone
    const isEmail = accountType === 'email'
    
    if (!identifier) {
      showToast(`请输入${isEmail ? '邮箱' : '手机号'}`, 'error')
      return
    }
    
    if (isEmail && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
      showToast('请输入正确的邮箱地址', 'error')
      return
    }
    
    if (!isEmail && !/^1[3-9]\d{9}$/.test(phone)) {
      showToast('请输入正确的手机号', 'error')
      return
    }
    
    setSendingCode(true)
    try {
      const res = await fetch(`${API_BASE}/api/user/send-code`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(isEmail ? { email } : { phone })
      })
      
      const data = await res.json()
      if (data.success) {
        setCountdown(60)
        showToast(data.testCode ? `测试验证码: ${data.testCode}` : '验证码已发送')
      } else {
        showToast(data.message, 'error')
      }
    } catch (e) {
      showToast(`网络错误: ${e.message}`, 'error')
    } finally {
      setSendingCode(false)
    }
  }

  // 自动登录函数
  const autoLogin = async (identifier, pwd) => {
    try {
      const isEmail = identifier.includes('@')
      const res = await fetch(`${API_BASE}/api/user/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          email: isEmail ? identifier : null,
          phone: isEmail ? null : identifier,
          password: pwd
        })
      })
      
      const data = await res.json()
      if (data.success) {
        const userData = data.data.user
        localStorage.setItem('kiro_user', JSON.stringify(userData))
        localStorage.setItem('kiro_token', data.data.token)
        onVerified({
          code: userData.email || userData.phone,
          nickname: userData.nickname,
          remaining_quota: userData.is_vip ? -1 : 0,
          total_quota: userData.is_vip ? -1 : 0,
          expires_at: userData.vip_expires_at || null,
          is_vip: userData.is_vip || false
        })
      }
    } catch (e) {
      console.error('自动登录失败:', e)
    }
  }

  const handleRegister = async () => {
    const identifier = accountType === 'email' ? email : phone
    if (!identifier || !code || !password) {
      showToast('请填写完整信息', 'error')
      return
    }
    if (password.length < 6) {
      showToast('密码至少6位', 'error')
      return
    }
    setLoading(true)
    try {
      const res = await fetch(`${API_BASE}/api/user/register`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          email: accountType === 'email' ? email : null,
          phone: accountType === 'phone' ? phone : null,
          code,
          password,
          nickname
        })
      })
      
      const data = await res.json()
      if (data.success) {
        showToast('注册成功，正在自动登录...')
        // 自动登录
        await autoLogin(identifier, password)
      } else {
        showToast(data.message, 'error')
      }
    } catch (e) {
      showToast(`网络错误: ${e.message}`, 'error')
    } finally {
      setLoading(false)
    }
  }

  // 验证码登录
  const handleCodeLogin = async () => {
    const identifier = accountType === 'email' ? email : phone
    if (!identifier || !code) {
      showToast('请输入账号和验证码', 'error')
      return
    }
    setLoading(true)
    try {
      const res = await fetch(`${API_BASE}/api/user/login-code`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          email: accountType === 'email' ? email : null,
          phone: accountType === 'phone' ? phone : null,
          code
        })
      })
      
      const data = await res.json()
      if (data.success) {
        const userData = data.data.user
        localStorage.setItem('kiro_user', JSON.stringify(userData))
        localStorage.setItem('kiro_token', data.data.token)
        showToast('登录成功')
        onVerified({
          code: userData.email || userData.phone,
          nickname: userData.nickname,
          remaining_quota: userData.is_vip ? -1 : 0,
          total_quota: userData.is_vip ? -1 : 0,
          expires_at: userData.vip_expires_at || null,
          is_vip: userData.is_vip || false
        })
      } else {
        showToast(data.message, 'error')
      }
    } catch (e) {
      showToast(`网络错误: ${e.message}`, 'error')
    } finally {
      setLoading(false)
    }
  }

  // 密码登录
  const handlePasswordLogin = async () => {
    const identifier = accountType === 'email' ? email : phone
    if (!identifier || !password) {
      showToast('请输入账号和密码', 'error')
      return
    }
    setLoading(true)
    try {
      const res = await fetch(`${API_BASE}/api/user/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          email: accountType === 'email' ? email : null,
          phone: accountType === 'phone' ? phone : null,
          password
        })
      })
      
      const data = await res.json()
      if (data.success) {
        const userData = data.data.user
        localStorage.setItem('kiro_user', JSON.stringify(userData))
        localStorage.setItem('kiro_token', data.data.token)
        showToast('登录成功')
        onVerified({
          code: userData.email || userData.phone,
          nickname: userData.nickname,
          remaining_quota: userData.is_vip ? -1 : 0,
          total_quota: userData.is_vip ? -1 : 0,
          expires_at: userData.vip_expires_at || null,
          is_vip: userData.is_vip || false
        })
      } else {
        showToast(data.message, 'error')
      }
    } catch (e) {
      showToast(`网络错误: ${e.message}`, 'error')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="h-full flex items-center justify-center p-8 relative">
      {/* Toast 提示 */}
      {toast.show && (
        <div className={`fixed bottom-6 right-6 z-50 px-5 py-3 rounded-xl shadow-2xl animate-slide-up ${
          toast.type === 'success' 
            ? 'bg-emerald-500/90 text-white' 
            : 'bg-red-500/90 text-white'
        }`}>
          <div className="flex items-center gap-2">
            {toast.type === 'success' ? <CheckCircle size={18} /> : <Shield size={18} />}
            <span className="text-sm font-medium">{toast.message}</span>
          </div>
        </div>
      )}

      <div className="w-full max-w-md glass-card p-8 animate-scale-in">
        {/* 返回按钮 */}
        {onBack && (
          <button
            onClick={onBack}
            className="mb-6 flex items-center gap-2 text-slate-400 hover:text-white transition-colors"
          >
            <ArrowLeft size={18} />
            <span className="text-sm">返回</span>
          </button>
        )}
        
        {/* Logo */}
        <div className="text-center mb-8">
          <div className="w-16 h-16 bg-gradient-to-br from-indigo-500 to-purple-600 rounded-2xl flex items-center justify-center mx-auto mb-4 shadow-xl shadow-indigo-500/30">
            <Shield size={32} className="text-white" />
          </div>
          <h1 className="text-2xl font-bold text-white">
            {mode === 'login' ? '登录账号' : '注册账号'}
          </h1>
          <p className="text-slate-500 mt-1 text-sm">
            {mode === 'login' 
              ? (loginMethod === 'code' ? '使用验证码快速登录' : '使用密码登录')
              : '创建新账号开始使用'}
          </p>
        </div>

        {/* 账号类型切换 */}
        <div className="flex gap-2 mb-6 p-1 rounded-xl bg-white/5">
          <button
            onClick={() => setAccountType('email')}
            className={`flex-1 py-2.5 rounded-lg text-sm font-medium transition-all ${
              accountType === 'email'
                ? 'bg-indigo-500 text-white shadow-lg'
                : 'text-slate-400 hover:text-white'
            }`}
          >
            <Mail size={16} className="inline mr-2" />
            邮箱
          </button>
          <button
            onClick={() => setAccountType('phone')}
            className={`flex-1 py-2.5 rounded-lg text-sm font-medium transition-all ${
              accountType === 'phone'
                ? 'bg-indigo-500 text-white shadow-lg'
                : 'text-slate-400 hover:text-white'
            }`}
          >
            <Phone size={16} className="inline mr-2" />
            手机号
          </button>
        </div>

        {/* 登录表单 - 验证码登录 */}
        {mode === 'login' && loginMethod === 'code' && (
          <div className="space-y-4">
            <div className="relative">
              {accountType === 'email' ? (
                <>
                  <Mail size={18} className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-500" />
                  <input
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    placeholder="邮箱地址"
                    className="input-aurora pl-12"
                  />
                </>
              ) : (
                <>
                  <Phone size={18} className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-500" />
                  <input
                    type="tel"
                    value={phone}
                    onChange={(e) => setPhone(e.target.value)}
                    placeholder="手机号"
                    maxLength={11}
                    className="input-aurora pl-12"
                  />
                </>
              )}
            </div>
            
            <div className="flex gap-3">
              <input
                type="text"
                value={code}
                onChange={(e) => setCode(e.target.value)}
                placeholder="验证码"
                maxLength={6}
                className="input-aurora flex-1"
              />
              <button
                onClick={handleSendCode}
                disabled={sendingCode || countdown > 0}
                className="btn-ghost whitespace-nowrap px-4"
              >
                {sendingCode ? <Loader2 size={16} className="animate-spin" /> : countdown > 0 ? `${countdown}s` : '获取验证码'}
              </button>
            </div>
            
            <button
              onClick={handleCodeLogin}
              disabled={loading}
              className="btn-aurora w-full flex items-center justify-center gap-2"
            >
              {loading ? <Loader2 size={18} className="animate-spin" /> : <ArrowRight size={18} />}
              <span>登录</span>
            </button>
            
            <div className="flex items-center justify-between text-sm">
              <button 
                onClick={() => setMode('register')} 
                className="text-slate-400 hover:text-indigo-400 transition-colors"
              >
                没有账号？立即注册
              </button>
              <button 
                onClick={() => setLoginMethod('password')} 
                className="text-slate-500 hover:text-slate-300 transition-colors flex items-center gap-1"
              >
                <KeyRound size={14} />
                密码登录
              </button>
            </div>
          </div>
        )}

        {/* 登录表单 - 密码登录 */}
        {mode === 'login' && loginMethod === 'password' && (
          <div className="space-y-4">
            <div className="relative">
              {accountType === 'email' ? (
                <>
                  <Mail size={18} className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-500" />
                  <input
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    placeholder="邮箱地址"
                    className="input-aurora pl-12"
                  />
                </>
              ) : (
                <>
                  <Phone size={18} className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-500" />
                  <input
                    type="tel"
                    value={phone}
                    onChange={(e) => setPhone(e.target.value)}
                    placeholder="手机号"
                    maxLength={11}
                    className="input-aurora pl-12"
                  />
                </>
              )}
            </div>
            
            <div className="relative">
              <Lock size={18} className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-500" />
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="密码"
                className="input-aurora pl-12"
              />
            </div>
            
            <button
              onClick={handlePasswordLogin}
              disabled={loading}
              className="btn-aurora w-full flex items-center justify-center gap-2"
            >
              {loading ? <Loader2 size={18} className="animate-spin" /> : <ArrowRight size={18} />}
              <span>登录</span>
            </button>
            
            <div className="flex items-center justify-between text-sm">
              <button 
                onClick={() => setMode('register')} 
                className="text-slate-400 hover:text-indigo-400 transition-colors"
              >
                没有账号？立即注册
              </button>
              <button 
                onClick={() => setLoginMethod('code')} 
                className="text-slate-500 hover:text-slate-300 transition-colors flex items-center gap-1"
              >
                <Mail size={14} />
                验证码登录
              </button>
            </div>
          </div>
        )}

        {/* 注册表单 */}
        {mode === 'register' && (
          <div className="space-y-4">
            <div className="relative">
              {accountType === 'email' ? (
                <>
                  <Mail size={18} className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-500" />
                  <input
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    placeholder="邮箱地址"
                    className="input-aurora pl-12"
                  />
                </>
              ) : (
                <>
                  <Phone size={18} className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-500" />
                  <input
                    type="tel"
                    value={phone}
                    onChange={(e) => setPhone(e.target.value)}
                    placeholder="手机号"
                    maxLength={11}
                    className="input-aurora pl-12"
                  />
                </>
              )}
            </div>
            
            <div className="flex gap-3">
              <input
                type="text"
                value={code}
                onChange={(e) => setCode(e.target.value)}
                placeholder="验证码"
                maxLength={6}
                className="input-aurora flex-1"
              />
              <button
                onClick={handleSendCode}
                disabled={sendingCode || countdown > 0}
                className="btn-ghost whitespace-nowrap px-4"
              >
                {sendingCode ? <Loader2 size={16} className="animate-spin" /> : countdown > 0 ? `${countdown}s` : '获取验证码'}
              </button>
            </div>
            
            <div className="relative">
              <User size={18} className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-500" />
              <input
                type="text"
                value={nickname}
                onChange={(e) => setNickname(e.target.value)}
                placeholder="昵称（选填）"
                className="input-aurora pl-12"
              />
            </div>
            
            <div className="relative">
              <Lock size={18} className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-500" />
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="设置密码（至少6位）"
                className="input-aurora pl-12"
              />
            </div>
            
            <button
              onClick={handleRegister}
              disabled={loading}
              className="btn-aurora w-full flex items-center justify-center gap-2"
            >
              {loading ? <Loader2 size={18} className="animate-spin" /> : <CheckCircle size={18} />}
              <span>注册</span>
            </button>
            
            <div className="text-center">
              <button 
                onClick={() => setMode('login')} 
                className="text-sm text-slate-400 hover:text-indigo-400 transition-colors"
              >
                已有账号？立即登录
              </button>
            </div>
          </div>
        )}

        {/* VIP 提示 */}
        <div className="mt-6 p-4 rounded-xl bg-amber-500/10 border border-amber-500/20">
          <div className="flex items-center gap-3">
            <Crown size={20} className="text-amber-400" />
            <div className="text-sm">
              <span className="text-amber-400 font-medium">开通会员</span>
              <span className="text-slate-400"> 可使用注册账号功能</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
