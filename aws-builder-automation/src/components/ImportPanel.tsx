import { useState } from 'react';
import { Upload, FileText, Loader2 } from 'lucide-react';
import { api } from '../api';
import { showWarning, showError } from '../utils/dialog';
import './ImportPanel.css';

interface ImportPanelProps {
  onImportComplete: () => void;
}

export function ImportPanel({ onImportComplete }: ImportPanelProps) {
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
    <div className="import-panel">
      <div className="import-header">
        <h3>数据导入</h3>
        <p className="import-description">
          格式: 邮箱地址----邮箱密码----客户端ID----refresh_token令牌
        </p>
      </div>

      <div className="import-body">
        <div className="import-textarea-container">
          <textarea
            className="import-textarea"
            placeholder="粘贴导入数据...&#10;&#10;示例:&#10;user@example.com----password123----client-id-here----refresh-token-here"
            value={content}
            onChange={e => setContent(e.target.value)}
            rows={10}
          />
        </div>

        <div className="import-actions">
          <button
            className="import-button import-button-primary"
            onClick={handlePasteImport}
            disabled={isLoading || !content.trim()}
          >
            {isLoading ? (
              <>
                <Loader2 size={18} className="spin" />
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
            className="import-button import-button-secondary"
            onClick={handleFileImport}
            disabled={isLoading}
          >
            {isLoading ? (
              <>
                <Loader2 size={18} className="spin" />
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
          <div className="import-result">
            <div className="import-result-header">
              <h4>导入结果</h4>
            </div>
            <div className="import-result-stats">
              <div className="import-stat import-stat-success">
                <span className="import-stat-label">成功:</span>
                <span className="import-stat-value">{result.success_count}</span>
              </div>
              <div className="import-stat import-stat-error">
                <span className="import-stat-label">失败:</span>
                <span className="import-stat-value">{result.error_count}</span>
              </div>
            </div>

            {result.errors && result.errors.length > 0 && (
              <div className="import-errors">
                <h5>错误详情:</h5>
                <div className="import-errors-list">
                  {result.errors.map((error: any, index: number) => (
                    <div key={index} className="import-error-item">
                      <div className="import-error-line">
                        第 {error.line_number} 行
                      </div>
                      <div className="import-error-reason">
                        {error.reason}
                      </div>
                      <div className="import-error-content">
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
