import { useState } from 'react';
import { Upload, FileText, Loader2 } from 'lucide-react';
import { api } from '../../api/autoRegister';
import { showWarning, showError } from '../../utils/dialog';
import { useTheme } from '../../contexts/ThemeContext';

interface ImportPanelProps {
  onImportComplete: () => void;
}

export function ImportPanel({ onImportComplete }: ImportPanelProps) {
  const { colors } = useTheme();
  const [content, setContent] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [result, setResult] = useState<any>(null);

  const handlePasteImport = async () => {
    if (!content.trim()) {
      await showWarning('请输入或粘贴导入内容');
      return;
    }

    setIsLoading(true);
    setResult(null);

    try {
      const importResult = await api.importAccounts(content);
      setResult(importResult);

      if (importResult.success_count > 0) {
        onImportComplete();
        setContent('');
      }
    } catch (error) {
      await showError('导入失败: ' + error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleFileImport = async () => {
    setIsLoading(true);
    setResult(null);

    try {
      const fileContent = await api.selectFile();

      if (fileContent) {
        const importResult = await api.importAccounts(fileContent);
        setResult(importResult);

        if (importResult.success_count > 0) {
          onImportComplete();
        }
      }
    } catch (error) {
      await showError('导入失败: ' + error);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className={`card-glow ${colors.card} rounded-2xl border ${colors.cardBorder} overflow-hidden shadow-sm`}>
      <div className={`px-6 py-5 border-b ${colors.cardBorder}`}>
        <h3 className={`text-lg font-semibold ${colors.text} mb-2`}>数据导入</h3>
        <p className={`text-sm ${colors.textMuted} font-mono`}>
          格式: 邮箱地址----邮箱密码----客户端ID----refresh_token令牌
        </p>
      </div>

      <div className="p-6">
        <div className="mb-5">
          <textarea
            className={`w-full min-h-[200px] px-4 py-3 border rounded-xl ${colors.text} ${colors.input} ${colors.inputFocus} focus:ring-2 font-mono text-sm resize-y transition-all`}
            placeholder="粘贴导入数据...&#10;&#10;示例:&#10;user@example.com----password123----client-id-here----refresh-token-here"
            value={content}
            onChange={e => setContent(e.target.value)}
            rows={10}
          />
        </div>

        <div className="flex gap-3 mb-5">
          <button
            className="flex-1 px-5 py-3 bg-blue-500 text-white rounded-xl font-medium shadow-sm hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center justify-center gap-2"
            onClick={handlePasteImport}
            disabled={isLoading || !content.trim()}
          >
            {isLoading ? (
              <>
                <Loader2 size={18} className="animate-spin" />
                导入中...
              </>
            ) : (
              <>
                <FileText size={18} />
                从文本导入
              </>
            )}
          </button>

          <button
            className={`flex-1 px-5 py-3 border rounded-xl font-medium shadow-sm hover:scale-[1.02] disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center justify-center gap-2 ${colors.text} ${colors.card} ${colors.cardBorder}`}
            onClick={handleFileImport}
            disabled={isLoading}
          >
            {isLoading ? (
              <>
                <Loader2 size={18} className="animate-spin" />
                导入中...
              </>
            ) : (
              <>
                <Upload size={18} />
                从文件导入
              </>
            )}
          </button>
        </div>

        {result && (
          <div className={`${colors.card} rounded-xl p-5 border ${colors.cardBorder}`}>
            <h4 className={`text-base font-semibold ${colors.text} mb-4`}>导入结果</h4>
            
            <div className="flex gap-5 mb-5">
              <div className="flex items-center gap-2 px-4 py-3 bg-green-500/10 rounded-lg">
                <span className={`text-sm font-medium ${colors.textMuted}`}>成功:</span>
                <span className="text-xl font-semibold text-green-500">{result.success_count}</span>
              </div>
              <div className="flex items-center gap-2 px-4 py-3 bg-red-500/10 rounded-lg">
                <span className={`text-sm font-medium ${colors.textMuted}`}>失败:</span>
                <span className="text-xl font-semibold text-red-500">{result.error_count}</span>
              </div>
            </div>

            {result.errors && result.errors.length > 0 && (
              <div className="mt-5">
                <h5 className={`text-sm font-semibold ${colors.text} mb-3`}>错误详情:</h5>
                <div className="max-h-[300px] overflow-y-auto space-y-2">
                  {result.errors.map((error: any, index: number) => (
                    <div key={index} className={`${colors.card} border ${colors.cardBorder} rounded-lg p-3`}>
                      <div className="text-xs font-semibold text-red-500 mb-1">
                        第 {error.line_number} 行
                      </div>
                      <div className={`text-sm ${colors.text} mb-1`}>
                        {error.reason}
                      </div>
                      <div className={`text-xs font-mono ${colors.textMuted} ${colors.card} p-2 rounded break-all`}>
                        {error.content}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
