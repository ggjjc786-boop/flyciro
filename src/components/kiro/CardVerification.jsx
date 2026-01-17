import { useState } from 'react'
import { useTheme } from '../contexts/ThemeContext'

export default function CardVerification({ onVerified, errorMessage }) {
  const [cardCode, setCardCode] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState(errorMessage || '')
  const { colors } = useTheme()

  const handleVerify = async () => {
    if (!cardCode.trim()) {
      setError('请输入卡密')
      return
    }

    setLoading(true)
    setError('')

    try {
      const response = await fetch('https://pe.xphdfs.me/api/card/check', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ code: cardCode.trim() })
      })

      const data = await response.json()

      if (data.success) {
        // 保存卡密信息到本地
        localStorage.setItem('verified_card', JSON.stringify({
          code: cardCode.trim(),
          quota: data.data.remaining_quota === '无限' ? -1 : data.data.remaining_quota,
          total_quota: data.data.total_quota === '无限' ? -1 : data.data.total_quota,
          expires_at: data.data.expires_at === '永久有效' ? null : data.data.expires_at,
          verified_at: new Date().toISOString()
        }))
        onVerified(data.data)
      } else {
        setError(data.message || '卡密验证失败')
      }
    } catch (err) {
      setError('网络错误，请检查连接')
      console.error('验证失败:', err)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className={`min-h-screen flex items-center justify-center ${colors.main}`}>
      <div className={`w-full max-w-md p-8 rounded-2xl ${colors.card} shadow-2xl`}>
        <div className="text-center mb-8">
          <h1 className={`text-3xl font-bold ${colors.text} mb-2`}>Kiro Account Manager</h1>
          <p className={colors.textMuted}>请输入卡密以使用软件</p>
        </div>

        <div className="space-y-4">
          {errorMessage && (
            <div className="p-4 rounded-lg bg-red-500/10 border border-red-500/20">
              <div className="flex items-center gap-2 mb-2">
                <span className="text-red-500 text-lg">⚠️</span>
                <span className="text-red-500 font-medium">卡密失效</span>
              </div>
              <p className="text-red-500 text-sm">{errorMessage}</p>
            </div>
          )}

          <div>
            <label className={`block text-sm font-medium ${colors.text} mb-2`}>
              卡密
            </label>
            <input
              type="text"
              value={cardCode}
              onChange={(e) => setCardCode(e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && handleVerify()}
              placeholder="请输入您的卡密"
              disabled={loading}
              className={`w-full px-4 py-3 rounded-lg ${colors.input} ${colors.text} border ${colors.cardBorder} focus:outline-none focus:ring-2 focus:ring-blue-500`}
            />
          </div>

          {error && (
            <div className="p-3 rounded-lg bg-red-500/10 border border-red-500/20">
              <p className="text-red-500 text-sm">{error}</p>
            </div>
          )}

          <button
            onClick={handleVerify}
            disabled={loading}
            className={`w-full py-3 rounded-lg font-medium transition-colors ${
              loading
                ? 'bg-gray-400 cursor-not-allowed'
                : 'bg-blue-500 hover:bg-blue-600 text-white'
            }`}
          >
            {loading ? '验证中...' : '验证卡密'}
          </button>
        </div>

        <div className={`mt-6 text-center text-sm ${colors.textMuted}`}>
          <p>首次使用请联系管理员获取卡密</p>
        </div>
      </div>
    </div>
  )
}
