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
    <div className={`card-glow ${colors.card} rounded-2xl border ${colors.cardBorder} overflow-hidden shadow-sm flex flex-col`}>
      <div className={`px-6 py-4 border-b ${colors.cardBorder} flex-shrink-0`}>
        <h3 className={`text-base font-semibold ${colors.text} mb-1`}>数据导入</h3>
        <p className={`text-xs ${colors.textMuted} font-mono`}>
          格式: 邮箱----密码----客户端ID----令牌
        </p>
      </div>

      <div className="p-4 flex-1 flex flex-col min-h-0">
        <div className="mb-3 flex-shrink-0">
          <textarea
            className={`w-full h-32 px-3 py-2 border rounded-lg ${colors.text} ${colors.input} ${colors.inputFocus} focus:ring-2 font-mono text-xs resize-none transition-all`}
            placeholder="粘贴导入数据...&#10;&#10;示例:&#10;user@example.com----password123----client-id----token"
            value={content}
            onChange={e => setContent(e.target.value)}
          />
        </div>

        <div className="flex gap-2 mb-3 flex-shrink-0">
          <button
            className="flex-1 px-3 py-2 bg-blue-500 text-white rounded-lg text-sm font-medium shadow-sm hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center justify-center gap-2"
            onClick={handlePasteImport}
            disabled={isLoading || !content.trim()}
          >
            {isLoading ? (
              <>
                <Loader2 size={16} className="animate-spin" />
                <span className="text-xs">导入中</span>
              </>
            ) : (
              <>
                <FileText size={16} />
                <span className="text-xs">文本导入</span>
              </>
            )}
          </button>

          <button
            className={`flex-1 px-3 py-2 border rounded-lg text-sm font-medium shadow-sm hover:scale-[1.02] disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center justify-center gap-2 ${colors.text} ${colors.card} ${colors.cardBorder}`}
            onClick={handleFileImport}
            disabled={isLoading}
          >
            {isLoading ? (
              <>
                <Loader2 size={16} className="animate-spin" />
                <span className="text-xs">导入中</span>
              </>
            ) : (
              <>
                <Upload size={16} />
                <span className="text-xs">文件导入</span>
              </>
            )}
          </button>
        </div>

        {result && (
          <div className={`${colors.card} rounded-lg p-3 border ${colors.cardBorder} flex-shrink-0 overflow-y-auto max-h-64`}>
            <h4 className={`text-sm font-semibold ${colors.text} mb-2`}>导入结果</h4>
            
            <div className="flex gap-3 mb-3">
              <div className="flex items-center gap-1 px-2 py-1 bg-green-500/10 rounded text-xs">
                <span className={colors.textMuted}>成功:</span>
                <span className="font-semibold text-green-500">{result.success_count}</span>
              </div>
              <div className="flex items-center gap-1 px-2 py-1 bg-red-500/10 rounded text-xs">
                <span className={colors.textMuted}>失败:</span>
                <span className="font-semibold text-red-500">{result.error_count}</span>
              </div>
            </div>

            {result.errors && result.errors.length > 0 && (
              <div className="mt-3">
                <h5 className={`text-xs font-semibold ${colors.text} mb-2`}>错误详情:</h5>
                <div className="space-y-1 max-h-32 overflow-y-auto">
                  {result.errors.map((error: any, index: number) => (
                    <div key={index} className={`${colors.card} border ${colors.cardBorder} rounded p-2 text-xs`}>
                      <div className="font-semibold text-red-500 mb-0.5">
                        第 {error.line_number} 行
                      </div>
                      <div className={`${colors.text} mb-0.5 text-xs`}>
                        {error.reason}
                      </div>
                      <div className={`font-mono ${colors.textMuted} ${colors.card} p-1 rounded text-[10px] break-all`}>
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
