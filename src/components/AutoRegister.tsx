import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';

interface RegisterResult {
  success: boolean;
  email?: string;
  access_token?: string;
  refresh_token?: string;
  error?: string;
}

export const AutoRegister: React.FC = () => {
  const { t } = useTranslation();
  const [isRegistering, setIsRegistering] = useState(false);
  const [status, setStatus] = useState('');
  const [result, setResult] = useState<RegisterResult | null>(null);

  const handleAutoRegister = async () => {
    setIsRegistering(true);
    setStatus(t('autoRegister.starting'));
    setResult(null);

    try {
      // 调用 Rust 后端的自动注册命令
      const registerResult = await invoke<RegisterResult>('auto_register_kiro_account');
      
      setResult(registerResult);
      
      if (registerResult.success) {
        setStatus(t('autoRegister.success'));
        
        // 自动导入到账号列表
        if (registerResult.email && registerResult.access_token) {
          await invoke('import_registered_account', {
            email: registerResult.email,
            accessToken: registerResult.access_token,
            refreshToken: registerResult.refresh_token || '',
          });
          setStatus(t('autoRegister.imported'));
        }
      } else {
        setStatus(t('autoRegister.failed') + ': ' + (registerResult.error || 'Unknown error'));
      }
    } catch (error) {
      console.error('Auto register error:', error);
      setStatus(t('autoRegister.error') + ': ' + String(error));
    } finally {
      setIsRegistering(false);
    }
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
      <h2 className="text-xl font-semibold mb-4 text-gray-900 dark:text-white">
        {t('autoRegister.title')}
      </h2>
      
      <p className="text-gray-600 dark:text-gray-400 mb-4">
        {t('autoRegister.description')}
      </p>

      <button
        onClick={handleAutoRegister}
        disabled={isRegistering}
        className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {isRegistering ? t('autoRegister.registering') : t('autoRegister.start')}
      </button>

      {status && (
        <div className="mt-4 p-3 bg-gray-100 dark:bg-gray-700 rounded">
          <p className="text-sm text-gray-700 dark:text-gray-300">{status}</p>
        </div>
      )}

      {result && result.success && (
        <div className="mt-4 p-4 bg-green-50 dark:bg-green-900/20 rounded border border-green-200 dark:border-green-800">
          <h3 className="font-semibold text-green-800 dark:text-green-200 mb-2">
            {t('autoRegister.accountInfo')}
          </h3>
          <div className="space-y-1 text-sm">
            <p className="text-gray-700 dark:text-gray-300">
              <span className="font-medium">{t('autoRegister.email')}:</span> {result.email}
            </p>
            <p className="text-gray-700 dark:text-gray-300">
              <span className="font-medium">{t('autoRegister.token')}:</span> {result.access_token?.substring(0, 20)}...
            </p>
          </div>
        </div>
      )}
    </div>
  );
};
