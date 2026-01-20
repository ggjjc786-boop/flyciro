import { useState, useEffect } from 'react';
import { Key, Loader2, Shield, Smartphone, AlertCircle, CheckCircle } from 'lucide-react';
import { cardAuthApi } from '../api/cardAuth';
import { useTheme } from '../contexts/ThemeContext';

interface CardAuthProps {
  onAuthSuccess: (expireTime?: string) => void;
}

export function CardAuth({ onAuthSuccess }: CardAuthProps) {
  const { colors, theme } = useTheme();
  const isDark = theme === 'dark';
  const [cardKey, setCardKey] = useState('');
  const [notice, setNotice] = useState('加载中...');
  const [deviceCode, setDeviceCode] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [showUnbind, setShowUnbind] = useState(false);
  const [errorMessage, setErrorMessage] = useState('');
  const [successMessage, setSuccessMessage] = useState('');

  useEffect(() => {
    // 加载公告
    cardAuthApi.getNotice()
      .then(setNotice)
      .catch(() => setNotice('无法加载公告'));

    // 加载设备码
    cardAuthApi.getDeviceCode()
      .then(setDeviceCode)
      .catch(() => setDeviceCode('无法获取'));
  }, []);

  const handleLogin = async () => {
    if (!cardKey.trim()) {
      setErrorMessage('请输入卡密');
      return;
    }

    setIsLoading(true);
    setErrorMessage('');
    setSuccessMessage('');
    
    try {
      const result = await cardAuthApi.verifyCardKey(cardKey);
      setSuccessMessage(`登录成功！${result.message}`);
      setTimeout(() => {
        onAuthSuccess(result.expire_time);
      }, 1500);
    } catch (error) {
      setErrorMessage(String(error));
    } finally {
      setIsLoading(false);
    }
  };

  const handleUnbind = async () => {
    if (!cardKey.trim()) {
      setErrorMessage('请输入卡密');
      return;
    }

    setIsLoading(true);
    setErrorMessage('');
    setSuccessMessage('');
    
    try {
      const message = await cardAuthApi.unbindCardKey(cardKey);
      setSuccessMessage(`解绑成功！${message}`);
      setCardKey('');
    } catch (error) {
      setErrorMessage(String(error));
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className={`min-h-screen ${colors.main} flex items-center justify-center p-4`}>
      <div className={`w-full max-w-md ${colors.card} rounded-2xl shadow-2xl border ${colors.cardBorder} overflow-hidden`}>
        {/* 头部 */}
        <div className="bg-gradient-to-r from-blue-500 to-purple-600 p-6 text-white">
          <div className="flex items-center justify-center mb-2">
            <Shield size={48} />
          </div>
          <h1 className="text-2xl font-bold text-center">卡密验证</h1>
          <p className="text-center text-sm opacity-90 mt-2">请输入您的卡密以使用自动注册功能</p>
        </div>

        {/* 内容 */}
        <div className="p-6 space-y-6">
          {/* 错误消息 */}
          {errorMessage && (
            <div className="bg-red-50 border border-red-200 rounded-xl p-4 flex items-start gap-3 animate-fade-in">
              <AlertCircle size={20} className="text-red-500 flex-shrink-0 mt-0.5" />
              <div className="flex-1">
                <h4 className="text-sm font-semibold text-red-800 mb-1">验证失败</h4>
                <p className="text-sm text-red-600">{errorMessage}</p>
              </div>
            </div>
          )}

          {/* 成功消息 */}
          {successMessage && (
            <div className="bg-green-50 border border-green-200 rounded-xl p-4 flex items-start gap-3 animate-fade-in">
              <CheckCircle size={20} className="text-green-500 flex-shrink-0 mt-0.5" />
              <div className="flex-1">
                <h4 className="text-sm font-semibold text-green-800 mb-1">操作成功</h4>
                <p className="text-sm text-green-600">{successMessage}</p>
              </div>
            </div>
          )}

          {/* 公告 */}
          <div className={`${isDark ? 'bg-blue-500/10' : 'bg-blue-50'} border ${isDark ? 'border-blue-500/20' : 'border-blue-200'} rounded-xl p-4`}>
            <h3 className={`text-sm font-semibold ${colors.text} mb-2 flex items-center gap-2`}>
              <span className="text-blue-500">📢</span> 公告
            </h3>
            <p className={`text-sm ${colors.textMuted} whitespace-pre-wrap`}>{notice}</p>
          </div>

          {/* 设备码 */}
          <div className={`${isDark ? 'bg-gray-500/10' : 'bg-gray-50'} border ${colors.cardBorder} rounded-xl p-4`}>
            <div className="flex items-center gap-2 mb-2">
              <Smartphone size={16} className={colors.textMuted} />
              <span className={`text-sm font-semibold ${colors.text}`}>设备码</span>
            </div>
            <p className={`text-xs ${colors.textMuted} font-mono break-all`}>{deviceCode}</p>
          </div>

          {/* 卡密输入 */}
          <div>
            <label className={`block text-sm font-medium ${colors.text} mb-2`}>
              卡密
            </label>
            <div className="relative">
              <Key size={18} className={`absolute left-3 top-1/2 -translate-y-1/2 ${colors.textMuted}`} />
              <input
                type="text"
                value={cardKey}
                onChange={(e) => setCardKey(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && handleLogin()}
                placeholder="请输入卡密"
                className={`w-full pl-10 pr-4 py-3 border rounded-xl ${colors.input} ${colors.inputFocus} ${colors.text} transition-all`}
                disabled={isLoading}
              />
            </div>
          </div>

          {/* 按钮 */}
          <div className="space-y-3">
            <button
              onClick={handleLogin}
              disabled={isLoading}
              className="w-full bg-gradient-to-r from-blue-500 to-purple-600 text-white py-3 rounded-xl font-medium shadow-lg hover:shadow-xl transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
            >
              {isLoading ? (
                <>
                  <Loader2 size={18} className="animate-spin" />
                  验证中...
                </>
              ) : (
                <>
                  <Key size={18} />
                  登录验证
                </>
              )}
            </button>

            <button
              onClick={() => setShowUnbind(!showUnbind)}
              className={`w-full ${colors.text} py-2 rounded-xl text-sm transition-all hover:bg-gray-100 ${isDark ? 'hover:bg-white/5' : ''}`}
            >
              {showUnbind ? '隐藏解绑' : '需要解绑？'}
            </button>

            {showUnbind && (
              <button
                onClick={handleUnbind}
                disabled={isLoading}
                className="w-full border-2 border-red-500 text-red-500 py-3 rounded-xl font-medium hover:bg-red-500/10 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
              >
                解绑卡密
              </button>
            )}
          </div>
        </div>

        {/* 底部 */}
        <div className={`${isDark ? 'bg-white/5' : 'bg-gray-50'} px-6 py-4 border-t ${colors.cardBorder}`}>
          <p className={`text-xs ${colors.textMuted} text-center`}>
            如有问题，请联系客服
          </p>
        </div>
      </div>
    </div>
  );
}
