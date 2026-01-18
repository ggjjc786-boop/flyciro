import { useState, useMemo } from 'react';
import { Trash2, Edit, Eye, Play, Loader2, Mail } from 'lucide-react';
import { Account } from '../../stores/autoRegisterStore';
import { api, EmailMessage } from '../../api/autoRegister';
import { showConfirm, showSuccess, showError } from '../../utils/dialog';
import { useTheme } from '../../contexts/ThemeContext';

interface AccountsTableProps {
  accounts: Account[];
  onRefresh: () => void;
}

export function AccountsTable({ accounts, onRefresh }: AccountsTableProps) {
  const { colors, theme } = useTheme();
  const isDark = theme === 'dark';
  const [search, setSearch] = useState('');
  const [sortField, setSortField] = useState<keyof Account>('created_at');
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('desc');
  const [currentPage, setCurrentPage] = useState(1);
  const [selectedAccount, setSelectedAccount] = useState<Account | null>(null);
  const [isDetailModalOpen, setIsDetailModalOpen] = useState(false);
  const [isEditModalOpen, setIsEditModalOpen] = useState(false);
  const [isEmailModalOpen, setIsEmailModalOpen] = useState(false);
  const [processingId, setProcessingId] = useState<number | null>(null);
  const [fetchingEmailId, setFetchingEmailId] = useState<number | null>(null);
  const [emails, setEmails] = useState<EmailMessage[]>([]);

  // 调试：组件加载时输出账号信息
  console.log('AccountsTable rendered with accounts:', accounts);
  console.log('Total accounts:', accounts.length);

  const itemsPerPage = 20;

  const filteredAccounts = useMemo(() => {
    return accounts.filter(account =>
      account.email.toLowerCase().includes(search.toLowerCase()) ||
      account.status.toLowerCase().includes(search.toLowerCase())
    );
  }, [accounts, search]);

  const sortedAccounts = useMemo(() => {
    return [...filteredAccounts].sort((a, b) => {
      const aValue = a[sortField];
      const bValue = b[sortField];

      if (aValue === null || aValue === undefined) return 1;
      if (bValue === null || bValue === undefined) return -1;

      if (typeof aValue === 'string' && typeof bValue === 'string') {
        return sortDirection === 'asc'
          ? aValue.localeCompare(bValue)
          : bValue.localeCompare(aValue);
      }

      return sortDirection === 'asc'
        ? (aValue > bValue ? 1 : -1)
        : (bValue > aValue ? 1 : -1);
    });
  }, [filteredAccounts, sortField, sortDirection]);

  const paginatedAccounts = useMemo(() => {
    const start = (currentPage - 1) * itemsPerPage;
    const end = start + itemsPerPage;
    return sortedAccounts.slice(start, end);
  }, [sortedAccounts, currentPage]);

  const totalPages = Math.ceil(sortedAccounts.length / itemsPerPage);

  const handleSort = (field: keyof Account) => {
    if (sortField === field) {
      setSortDirection(sortDirection === 'asc' ? 'desc' : 'asc');
    } else {
      setSortField(field);
      setSortDirection('asc');
    }
  };

  const handleDelete = async (id: number) => {
    console.log('=== DELETE BUTTON CLICKED ===');
    console.log('[Delete] Starting delete for account ID:', id);
    alert('删除按钮被点击了！ID: ' + id); // 临时测试
    try {
      const confirmed = await showConfirm('确定要删除这条记录吗?', '确认删除');
      console.log('[Delete] User confirmation:', confirmed);
      if (confirmed) {
        console.log('[Delete] Calling API to delete account:', id);
        await api.deleteAccount(id);
        console.log('[Delete] Delete successful, refreshing list');
        await showSuccess('删除成功');
        onRefresh();
      } else {
        console.log('[Delete] User cancelled deletion');
      }
    } catch (error) {
      console.error('[Delete] Error during deletion:', error);
      await showError('删除失败: ' + error);
    }
  };

  const handleStartRegistration = async (id: number) => {
    if (processingId) {
      return;
    }

    setProcessingId(id);
    try {
      await api.startRegistration(id);
      onRefresh();
    } catch (error) {
      onRefresh();
    } finally {
      setProcessingId(null);
    }
  };

  const handleFetchEmails = async (account: Account) => {
    if (fetchingEmailId) {
      return;
    }

    setFetchingEmailId(account.id);
    try {
      const emailList = await api.fetchEmails(account.id, 10);
      setEmails(emailList);
      setSelectedAccount(account);
      setIsEmailModalOpen(true);
    } catch (error) {
      await showError('获取邮件失败: ' + error);
    } finally {
      setFetchingEmailId(null);
    }
  };

  const getStatusText = (status: string) => {
    const statusMap: Record<string, string> = {
      not_registered: '未注册',
      in_progress: '进行中',
      registered: '已注册',
      error: '异常',
    };
    return statusMap[status] || status;
  };

  const getStatusClass = (status: string) => {
    const statusMap: Record<string, string> = {
      not_registered: isDark ? 'bg-blue-500/20 text-blue-400' : 'bg-blue-100 text-blue-700',
      in_progress: isDark ? 'bg-yellow-500/20 text-yellow-400' : 'bg-yellow-100 text-yellow-700',
      registered: isDark ? 'bg-green-500/20 text-green-400' : 'bg-green-100 text-green-700',
      error: isDark ? 'bg-red-500/20 text-red-400' : 'bg-red-100 text-red-700',
    };
    return statusMap[status] || (isDark ? 'bg-gray-500/20 text-gray-400' : 'bg-gray-100 text-gray-700');
  };

  return (
    <div className={`flex flex-col h-full ${colors.card}`}>
      {/* 测试按钮 */}
      <div className="px-6 py-2 bg-yellow-100 border-b border-yellow-300">
        <button
          onClick={() => {
            alert('测试按钮工作正常！');
            console.log('Test button clicked!');
          }}
          className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
        >
          🧪 点击测试按钮功能
        </button>
        <span className="ml-4 text-sm text-gray-700">
          如果这个按钮能弹出 alert，说明按钮功能正常
        </span>
      </div>
      
      <div className={`flex items-center justify-between px-6 py-4 border-b ${colors.cardBorder}`}>
        <input
          type="text"
          placeholder="搜索邮箱或状态..."
          value={search}
          onChange={(e) => {
            setSearch(e.target.value);
            setCurrentPage(1);
          }}
          className={`min-w-[300px] px-4 py-2 border rounded-xl ${colors.text} ${colors.input} ${colors.inputFocus} focus:ring-2 transition-all`}
        />
        <div className={`text-sm ${colors.textMuted}`}>
          共 {sortedAccounts.length} 条记录
        </div>
      </div>

      <div className="flex-1 overflow-auto">
        <table className="min-w-full">
          <thead className={`sticky top-0 ${colors.card} border-b ${colors.cardBorder}`}>
            <tr>
              <th onClick={() => handleSort('id')} className={`px-4 py-3 text-left text-sm font-semibold ${colors.text} cursor-pointer select-none hover:bg-opacity-50 transition-colors`}>
                序号 {sortField === 'id' && (sortDirection === 'asc' ? '↑' : '↓')}
              </th>
              <th onClick={() => handleSort('email')} className={`px-4 py-3 text-left text-sm font-semibold ${colors.text} cursor-pointer select-none hover:bg-opacity-50 transition-colors`}>
                注册邮箱 {sortField === 'email' && (sortDirection === 'asc' ? '↑' : '↓')}
              </th>
              <th className={`px-4 py-3 text-left text-sm font-semibold ${colors.text}`}>邮箱密码</th>
              <th onClick={() => handleSort('status')} className={`px-4 py-3 text-left text-sm font-semibold ${colors.text} cursor-pointer select-none hover:bg-opacity-50 transition-colors`}>
                状态 {sortField === 'status' && (sortDirection === 'asc' ? '↑' : '↓')}
              </th>
              <th className={`px-4 py-3 text-left text-sm font-semibold ${colors.text}`}>异常原因</th>
              <th className={`px-4 py-3 text-left text-sm font-semibold ${colors.text}`}>操作</th>
            </tr>
          </thead>
          <tbody>
            {paginatedAccounts.map((account, index) => (
              <tr key={account.id} className={`border-b ${colors.cardBorder} hover:bg-opacity-50 transition-colors`}>
                <td className={`px-4 py-3 text-sm ${colors.text}`}>{(currentPage - 1) * itemsPerPage + index + 1}</td>
                <td className={`px-4 py-3 text-sm font-mono ${colors.text}`}>{account.email}</td>
                <td className={`px-4 py-3 text-sm ${colors.textMuted}`}>
                  <span className="font-mono">••••••••</span>
                </td>
                <td className="px-4 py-3 text-sm">
                  <span className={`inline-block px-3 py-1 rounded-lg text-xs font-medium ${getStatusClass(account.status)}`}>
                    {getStatusText(account.status)}
                  </span>
                </td>
                <td className={`px-4 py-3 text-sm max-w-[200px]`}>
                  {account.error_reason && (
                    <span className="text-red-500 text-sm truncate block" title={account.error_reason}>
                      {account.error_reason.substring(0, 50)}
                      {account.error_reason.length > 50 && '...'}
                    </span>
                  )}
                </td>
                <td className="px-4 py-3">
                  <div className="flex gap-2">
                    <button
                      className={`w-8 h-8 flex items-center justify-center border rounded-lg transition-all hover:scale-110 ${colors.textMuted} ${colors.cardBorder} disabled:opacity-50`}
                      onClick={() => handleFetchEmails(account)}
                      title="获取邮件"
                      disabled={fetchingEmailId === account.id}
                    >
                      {fetchingEmailId === account.id ? (
                        <Loader2 size={16} className="animate-spin" />
                      ) : (
                        <Mail size={16} />
                      )}
                    </button>
                    <button
                      className={`w-8 h-8 flex items-center justify-center border rounded-lg transition-all hover:scale-110 ${colors.textMuted} ${colors.cardBorder}`}
                      onClick={() => {
                        console.log('VIEW DETAILS BUTTON CLICKED');
                        alert('查看详情按钮被点击了！');
                        setSelectedAccount(account);
                        setIsDetailModalOpen(true);
                      }}
                      title="查看详情"
                    >
                      <Eye size={16} />
                    </button>
                    <button
                      className={`w-8 h-8 flex items-center justify-center border rounded-lg transition-all hover:scale-110 ${colors.textMuted} ${colors.cardBorder} disabled:opacity-50`}
                      onClick={() => {
                        setSelectedAccount(account);
                        setIsEditModalOpen(true);
                      }}
                      title="编辑"
                      disabled={account.status === 'in_progress'}
                    >
                      <Edit size={16} />
                    </button>
                    {account.status === 'not_registered' && (
                      <button
                        className="w-8 h-8 flex items-center justify-center border border-blue-500 text-blue-500 rounded-lg transition-all hover:bg-blue-500/10 disabled:opacity-50"
                        onClick={() => handleStartRegistration(account.id)}
                        title="开始注册"
                        disabled={processingId === account.id || processingId !== null}
                      >
                        {processingId === account.id ? (
                          <Loader2 size={16} className="animate-spin" />
                        ) : (
                          <Play size={16} />
                        )}
                      </button>
                    )}
                    <button
                      className="w-8 h-8 flex items-center justify-center border border-red-500 text-red-500 rounded-lg transition-all hover:bg-red-500/10 disabled:opacity-50"
                      onClick={(e) => {
                        e.stopPropagation();
                        console.log('DELETE BUTTON CLICKED - Event:', e);
                        console.log('Account status:', account.status);
                        console.log('Button disabled:', account.status === 'in_progress');
                        handleDelete(account.id);
                      }}
                      title="删除"
                      disabled={false} // 临时移除禁用，用于测试
                      style={{ pointerEvents: 'auto', zIndex: 10 }} // 确保按钮可点击
                    >
                      <Trash2 size={16} />
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {totalPages > 1 && (
        <div className={`flex items-center justify-center gap-4 px-6 py-4 border-t ${colors.cardBorder}`}>
          <button
            onClick={() => setCurrentPage(p => Math.max(1, p - 1))}
            disabled={currentPage === 1}
            className={`px-4 py-2 border rounded-xl font-medium transition-all hover:scale-[1.02] disabled:opacity-50 ${colors.text} ${colors.card} ${colors.cardBorder}`}
          >
            上一页
          </button>
          <span className={`text-sm ${colors.textMuted}`}>
            第 {currentPage} / {totalPages} 页
          </span>
          <button
            onClick={() => setCurrentPage(p => Math.min(totalPages, p + 1))}
            disabled={currentPage === totalPages}
            className={`px-4 py-2 border rounded-xl font-medium transition-all hover:scale-[1.02] disabled:opacity-50 ${colors.text} ${colors.card} ${colors.cardBorder}`}
          >
            下一页
          </button>
        </div>
      )}

      {isDetailModalOpen && selectedAccount && (
        <DetailModal
          account={selectedAccount}
          onClose={() => {
            setIsDetailModalOpen(false);
            setSelectedAccount(null);
          }}
          colors={colors}
          isDark={isDark}
        />
      )}

      {isEditModalOpen && selectedAccount && (
        <EditModal
          account={selectedAccount}
          onClose={() => {
            setIsEditModalOpen(false);
            setSelectedAccount(null);
          }}
          onSave={() => {
            setIsEditModalOpen(false);
            setSelectedAccount(null);
            onRefresh();
          }}
          colors={colors}
          isDark={isDark}
        />
      )}

      {isEmailModalOpen && selectedAccount && (
        <EmailModal
          account={selectedAccount}
          emails={emails}
          onClose={() => {
            setIsEmailModalOpen(false);
            setSelectedAccount(null);
            setEmails([]);
          }}
          colors={colors}
          isDark={isDark}
        />
      )}
    </div>
  );
}

function DetailModal({ account, onClose, colors, isDark }: { account: Account; onClose: () => void; colors: any; isDark: boolean }) {
  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-[2000] animate-fade-in" onClick={onClose}>
      <div className={`${colors.card} rounded-2xl shadow-xl w-[90%] max-w-[600px] max-h-[90vh] overflow-auto animate-slide-up`} onClick={e => e.stopPropagation()}>
        <div className={`flex items-center justify-between px-6 py-5 border-b ${colors.cardBorder}`}>
          <h2 className={`text-xl font-semibold ${colors.text}`}>账号详情</h2>
          <button className={`w-8 h-8 flex items-center justify-center rounded-lg ${isDark ? 'hover:bg-white/10' : 'hover:bg-gray-100'} ${colors.textMuted} text-2xl transition-colors`} onClick={onClose}>×</button>
        </div>
        <div className="p-6 space-y-4">
          <div className={`flex py-2 border-b ${colors.cardBorder} last:border-0`}>
            <label className={`flex-shrink-0 w-32 font-medium ${colors.textMuted}`}>ID:</label>
            <span className={colors.text}>{account.id}</span>
          </div>
          <div className={`flex py-2 border-b ${colors.cardBorder} last:border-0`}>
            <label className={`flex-shrink-0 w-32 font-medium ${colors.textMuted}`}>注册邮箱:</label>
            <span className={colors.text}>{account.email}</span>
          </div>
          <div className={`flex py-2 border-b ${colors.cardBorder} last:border-0`}>
            <label className={`flex-shrink-0 w-32 font-medium ${colors.textMuted}`}>邮箱密码:</label>
            <span className={colors.text}>{account.email_password}</span>
          </div>
          <div className={`flex py-2 border-b ${colors.cardBorder} last:border-0`}>
            <label className={`flex-shrink-0 w-32 font-medium ${colors.textMuted}`}>客户端ID:</label>
            <span className={`font-mono text-sm ${colors.text}`}>{account.client_id}</span>
          </div>
          <div className={`flex py-2 border-b ${colors.cardBorder} last:border-0`}>
            <label className={`flex-shrink-0 w-32 font-medium ${colors.textMuted}`}>Refresh Token:</label>
            <span className={`font-mono text-sm break-all ${colors.text}`}>{account.refresh_token}</span>
          </div>
          {account.kiro_password && (
            <div className={`flex py-2 border-b ${colors.cardBorder} last:border-0`}>
              <label className={`flex-shrink-0 w-32 font-medium ${colors.textMuted}`}>Kiro密码:</label>
              <span className={colors.text}>{account.kiro_password}</span>
            </div>
          )}
          <div className={`flex py-2 border-b ${colors.cardBorder} last:border-0`}>
            <label className={`flex-shrink-0 w-32 font-medium ${colors.textMuted}`}>状态:</label>
            <span className={colors.text}>{account.status}</span>
          </div>
          {account.error_reason && (
            <div className={`flex py-2 border-b ${colors.cardBorder} last:border-0`}>
              <label className={`flex-shrink-0 w-32 font-medium ${colors.textMuted}`}>异常原因:</label>
              <span className="text-red-500">{account.error_reason}</span>
            </div>
          )}
          <div className={`flex py-2 border-b ${colors.cardBorder} last:border-0`}>
            <label className={`flex-shrink-0 w-32 font-medium ${colors.textMuted}`}>创建时间:</label>
            <span className={colors.text}>{new Date(account.created_at).toLocaleString('zh-CN')}</span>
          </div>
          <div className={`flex py-2 border-b ${colors.cardBorder} last:border-0`}>
            <label className={`flex-shrink-0 w-32 font-medium ${colors.textMuted}`}>更新时间:</label>
            <span className={colors.text}>{new Date(account.updated_at).toLocaleString('zh-CN')}</span>
          </div>
        </div>
      </div>
    </div>
  );
}

function EditModal({
  account,
  onClose,
  onSave,
  colors,
  isDark,
}: {
  account: Account;
  onClose: () => void;
  onSave: () => void;
  colors: any;
  isDark: boolean;
}) {
  const [formData, setFormData] = useState({
    email: account.email,
    email_password: account.email_password,
    client_id: account.client_id,
    refresh_token: account.refresh_token,
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await api.updateAccount({
        id: account.id,
        ...formData,
      });
      await showSuccess('更新成功');
      onSave();
    } catch (error) {
      await showError('更新失败: ' + error);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-[2000] animate-fade-in" onClick={onClose}>
      <div className={`${colors.card} rounded-2xl shadow-xl w-[90%] max-w-[600px] max-h-[90vh] overflow-auto animate-slide-up`} onClick={e => e.stopPropagation()}>
        <div className={`flex items-center justify-between px-6 py-5 border-b ${colors.cardBorder}`}>
          <h2 className={`text-xl font-semibold ${colors.text}`}>编辑账号</h2>
          <button className={`w-8 h-8 flex items-center justify-center rounded-lg ${isDark ? 'hover:bg-white/10' : 'hover:bg-gray-100'} ${colors.textMuted} text-2xl transition-colors`} onClick={onClose}>×</button>
        </div>
        <form onSubmit={handleSubmit}>
          <div className="p-6 space-y-5">
            <div>
              <label className={`block text-sm font-medium ${colors.textMuted} mb-2`}>注册邮箱:</label>
              <input
                type="email"
                value={formData.email}
                onChange={e => setFormData({ ...formData, email: e.target.value })}
                required
                className={`w-full px-4 py-3 border rounded-xl ${colors.text} ${colors.input} ${colors.inputFocus} focus:ring-2 transition-all`}
              />
            </div>
            <div>
              <label className={`block text-sm font-medium ${colors.textMuted} mb-2`}>邮箱密码:</label>
              <input
                type="text"
                value={formData.email_password}
                onChange={e => setFormData({ ...formData, email_password: e.target.value })}
                required
                className={`w-full px-4 py-3 border rounded-xl ${colors.text} ${colors.input} ${colors.inputFocus} focus:ring-2 transition-all`}
              />
            </div>
            <div>
              <label className={`block text-sm font-medium ${colors.textMuted} mb-2`}>客户端ID:</label>
              <input
                type="text"
                value={formData.client_id}
                onChange={e => setFormData({ ...formData, client_id: e.target.value })}
                required
                className={`w-full px-4 py-3 border rounded-xl ${colors.text} ${colors.input} ${colors.inputFocus} focus:ring-2 transition-all`}
              />
            </div>
            <div>
              <label className={`block text-sm font-medium ${colors.textMuted} mb-2`}>Refresh Token:</label>
              <textarea
                value={formData.refresh_token}
                onChange={e => setFormData({ ...formData, refresh_token: e.target.value })}
                required
                rows={3}
                className={`w-full px-4 py-3 border rounded-xl ${colors.text} ${colors.input} ${colors.inputFocus} focus:ring-2 resize-y transition-all`}
              />
            </div>
          </div>
          <div className={`flex items-center justify-end gap-3 px-6 py-4 border-t ${colors.cardBorder}`}>
            <button type="button" onClick={onClose} className={`px-5 py-2.5 border rounded-xl font-medium transition-all hover:scale-[1.02] ${colors.text} ${colors.card} ${colors.cardBorder}`}>
              取消
            </button>
            <button type="submit" className="px-5 py-2.5 bg-blue-500 text-white rounded-xl font-medium shadow-sm hover:bg-blue-600 transition-all">
              保存
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

function EmailModal({
  account,
  emails,
  onClose,
  colors,
  isDark,
}: {
  account: Account;
  emails: EmailMessage[];
  onClose: () => void;
  colors: any;
  isDark: boolean;
}) {
  const [selectedEmail, setSelectedEmail] = useState<EmailMessage | null>(null);

  const formatDateTime = (dateStr: string) => {
    return new Date(dateStr).toLocaleString('zh-CN');
  };

  const stripHtml = (html: string) => {
    const doc = new DOMParser().parseFromString(html, 'text/html');
    return doc.body.textContent || '';
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-[2000] animate-fade-in" onClick={onClose}>
      <div className={`${colors.card} rounded-2xl shadow-xl w-[90%] max-w-[800px] h-[80vh] flex flex-col animate-slide-up`} onClick={e => e.stopPropagation()}>
        <div className={`flex items-center justify-between px-6 py-5 border-b ${colors.cardBorder}`}>
          <h2 className={`text-xl font-semibold ${colors.text}`}>邮件列表 - {account.email}</h2>
          <button className={`w-8 h-8 flex items-center justify-center rounded-lg ${isDark ? 'hover:bg-white/10' : 'hover:bg-gray-100'} ${colors.textMuted} text-2xl transition-colors`} onClick={onClose}>×</button>
        </div>
        <div className="flex-1 overflow-auto p-0">
          {emails.length === 0 ? (
            <div className={`flex items-center justify-center h-[200px] ${colors.textMuted} text-base`}>暂无邮件</div>
          ) : selectedEmail ? (
            <div className="p-6">
              <button 
                className="text-blue-500 hover:underline text-sm mb-4" 
                onClick={() => setSelectedEmail(null)}
              >
                ← 返回列表
              </button>
              <div className={`mb-5 pb-4 border-b ${colors.cardBorder}`}>
                <h3 className={`text-lg font-semibold ${colors.text} mb-3`}>{selectedEmail.subject || '(无主题)'}</h3>
                <div className={`flex flex-col gap-1 text-sm ${colors.textMuted}`}>
                  <span>发件人: {selectedEmail.from_address}</span>
                  <span>时间: {formatDateTime(selectedEmail.received_datetime)}</span>
                </div>
              </div>
              <div 
                className={`text-sm leading-relaxed ${colors.text} break-words`}
                dangerouslySetInnerHTML={{ __html: selectedEmail.body_content }}
              />
            </div>
          ) : (
            <div className="flex flex-col">
              {emails.map((email) => (
                <div 
                  key={email.id} 
                  className={`px-6 py-4 border-b ${colors.cardBorder} cursor-pointer transition-colors hover:bg-opacity-50`}
                  onClick={() => setSelectedEmail(email)}
                >
                  <div className="flex items-center justify-between mb-1">
                    <span className={`font-medium ${colors.text} flex-1 truncate mr-4`}>{email.subject || '(无主题)'}</span>
                    <span className={`text-xs ${colors.textMuted} flex-shrink-0`}>{formatDateTime(email.received_datetime)}</span>
                  </div>
                  <div className={`text-sm ${colors.textMuted} mb-1`}>{email.from_address}</div>
                  <div className={`text-sm ${colors.textMuted} truncate`}>
                    {stripHtml(email.body_content).substring(0, 100)}...
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
