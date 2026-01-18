import { useState, useMemo } from 'react';
import { Trash2, Edit, Eye, Play, Loader2, Mail } from 'lucide-react';
import { Account } from '../store';
import { api, EmailMessage } from '../api';
import { showConfirm, showSuccess, showError } from '../utils/dialog';
import './AccountsTable.css';

interface AccountsTableProps {
  accounts: Account[];
  onRefresh: () => void;
}

export function AccountsTable({ accounts, onRefresh }: AccountsTableProps) {
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
    const confirmed = await showConfirm('确定要删除这条记录吗?', '确认删除');
    if (confirmed) {
      try {
        await api.deleteAccount(id);
        onRefresh();
      } catch (error) {
        await showError('删除失败: ' + error);
      }
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
    return `status-badge status-${status.replace('_', '-')}`;
  };

  return (
    <div className="accounts-table-container">
      <div className="table-header">
        <input
          type="text"
          placeholder="搜索邮箱或状态..."
          value={search}
          onChange={(e) => {
            setSearch(e.target.value);
            setCurrentPage(1);
          }}
          className="search-input"
        />
        <div className="table-stats">
          共 {sortedAccounts.length} 条记录
        </div>
      </div>

      <div className="table-wrapper">
        <table className="accounts-table">
          <thead>
            <tr>
              <th onClick={() => handleSort('id')}>
                序号 {sortField === 'id' && (sortDirection === 'asc' ? '↑' : '↓')}
              </th>
              <th onClick={() => handleSort('email')}>
                注册邮箱 {sortField === 'email' && (sortDirection === 'asc' ? '↑' : '↓')}
              </th>
              <th>邮箱密码</th>
              <th onClick={() => handleSort('status')}>
                状态 {sortField === 'status' && (sortDirection === 'asc' ? '↑' : '↓')}
              </th>
              <th>异常原因</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            {paginatedAccounts.map((account, index) => (
              <tr key={account.id}>
                <td>{(currentPage - 1) * itemsPerPage + index + 1}</td>
                <td className="email-cell">{account.email}</td>
                <td>
                  <span className="password-hidden">••••••••</span>
                </td>
                <td>
                  <span className={getStatusClass(account.status)}>
                    {getStatusText(account.status)}
                  </span>
                </td>
                <td className="error-cell">
                  {account.error_reason && (
                    <span className="error-text" title={account.error_reason}>
                      {account.error_reason.substring(0, 50)}
                      {account.error_reason.length > 50 && '...'}
                    </span>
                  )}
                </td>
                <td>
                  <div className="action-buttons">
                    <button
                      className="action-button"
                      onClick={() => handleFetchEmails(account)}
                      title="获取邮件"
                      disabled={fetchingEmailId === account.id}
                    >
                      {fetchingEmailId === account.id ? (
                        <Loader2 size={16} className="spin" />
                      ) : (
                        <Mail size={16} />
                      )}
                    </button>
                    <button
                      className="action-button"
                      onClick={() => {
                        setSelectedAccount(account);
                        setIsDetailModalOpen(true);
                      }}
                      title="查看详情"
                    >
                      <Eye size={16} />
                    </button>
                    <button
                      className="action-button"
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
                        className="action-button action-button-primary"
                        onClick={() => handleStartRegistration(account.id)}
                        title="开始注册"
                        disabled={processingId === account.id || processingId !== null}
                      >
                        {processingId === account.id ? (
                          <Loader2 size={16} className="spin" />
                        ) : (
                          <Play size={16} />
                        )}
                      </button>
                    )}
                    <button
                      className="action-button action-button-danger"
                      onClick={() => handleDelete(account.id)}
                      title="删除"
                      disabled={account.status === 'in_progress'}
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
        <div className="pagination">
          <button
            onClick={() => setCurrentPage(p => Math.max(1, p - 1))}
            disabled={currentPage === 1}
            className="pagination-button"
          >
            上一页
          </button>
          <span className="pagination-info">
            第 {currentPage} / {totalPages} 页
          </span>
          <button
            onClick={() => setCurrentPage(p => Math.min(totalPages, p + 1))}
            disabled={currentPage === totalPages}
            className="pagination-button"
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
        />
      )}
    </div>
  );
}

function DetailModal({ account, onClose }: { account: Account; onClose: () => void }) {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2>账号详情</h2>
          <button className="modal-close" onClick={onClose}>×</button>
        </div>
        <div className="modal-body">
          <div className="detail-item">
            <label>ID:</label>
            <span>{account.id}</span>
          </div>
          <div className="detail-item">
            <label>注册邮箱:</label>
            <span>{account.email}</span>
          </div>
          <div className="detail-item">
            <label>邮箱密码:</label>
            <span>{account.email_password}</span>
          </div>
          <div className="detail-item">
            <label>客户端ID:</label>
            <span className="monospace">{account.client_id}</span>
          </div>
          <div className="detail-item">
            <label>Refresh Token:</label>
            <span className="monospace break-all">{account.refresh_token}</span>
          </div>
          {account.kiro_password && (
            <div className="detail-item">
              <label>Kiro密码:</label>
              <span>{account.kiro_password}</span>
            </div>
          )}
          <div className="detail-item">
            <label>状态:</label>
            <span>{account.status}</span>
          </div>
          {account.error_reason && (
            <div className="detail-item">
              <label>异常原因:</label>
              <span className="error-text">{account.error_reason}</span>
            </div>
          )}
          <div className="detail-item">
            <label>创建时间:</label>
            <span>{new Date(account.created_at).toLocaleString('zh-CN')}</span>
          </div>
          <div className="detail-item">
            <label>更新时间:</label>
            <span>{new Date(account.updated_at).toLocaleString('zh-CN')}</span>
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
}: {
  account: Account;
  onClose: () => void;
  onSave: () => void;
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
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2>编辑账号</h2>
          <button className="modal-close" onClick={onClose}>×</button>
        </div>
        <form onSubmit={handleSubmit}>
          <div className="modal-body">
            <div className="form-group">
              <label>注册邮箱:</label>
              <input
                type="email"
                value={formData.email}
                onChange={e => setFormData({ ...formData, email: e.target.value })}
                required
              />
            </div>
            <div className="form-group">
              <label>邮箱密码:</label>
              <input
                type="text"
                value={formData.email_password}
                onChange={e => setFormData({ ...formData, email_password: e.target.value })}
                required
              />
            </div>
            <div className="form-group">
              <label>客户端ID:</label>
              <input
                type="text"
                value={formData.client_id}
                onChange={e => setFormData({ ...formData, client_id: e.target.value })}
                required
              />
            </div>
            <div className="form-group">
              <label>Refresh Token:</label>
              <textarea
                value={formData.refresh_token}
                onChange={e => setFormData({ ...formData, refresh_token: e.target.value })}
                required
                rows={3}
              />
            </div>
          </div>
          <div className="modal-footer">
            <button type="button" onClick={onClose} className="button-secondary">
              取消
            </button>
            <button type="submit" className="button-primary">
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
}: {
  account: Account;
  emails: EmailMessage[];
  onClose: () => void;
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
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content modal-large" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2>邮件列表 - {account.email}</h2>
          <button className="modal-close" onClick={onClose}>×</button>
        </div>
        <div className="modal-body email-modal-body">
          {emails.length === 0 ? (
            <div className="empty-state">暂无邮件</div>
          ) : selectedEmail ? (
            <div className="email-detail">
              <button 
                className="back-button" 
                onClick={() => setSelectedEmail(null)}
              >
                ← 返回列表
              </button>
              <div className="email-detail-header">
                <h3>{selectedEmail.subject || '(无主题)'}</h3>
                <div className="email-meta">
                  <span>发件人: {selectedEmail.from_address}</span>
                  <span>时间: {formatDateTime(selectedEmail.received_datetime)}</span>
                </div>
              </div>
              <div 
                className="email-body"
                dangerouslySetInnerHTML={{ __html: selectedEmail.body_content }}
              />
            </div>
          ) : (
            <div className="email-list">
              {emails.map((email) => (
                <div 
                  key={email.id} 
                  className="email-item"
                  onClick={() => setSelectedEmail(email)}
                >
                  <div className="email-item-header">
                    <span className="email-subject">{email.subject || '(无主题)'}</span>
                    <span className="email-time">{formatDateTime(email.received_datetime)}</span>
                  </div>
                  <div className="email-from">{email.from_address}</div>
                  <div className="email-preview">
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
