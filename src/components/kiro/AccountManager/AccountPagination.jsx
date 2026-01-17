import { ChevronsLeft, ChevronLeft, ChevronRight, ChevronsRight } from 'lucide-react'
import { useTheme } from '../../contexts/ThemeContext'
import { useI18n } from '../../i18n'

function AccountPagination({
  totalCount,
  pageSize,
  currentPage,
  totalPages,
  onPageSizeChange,
  onPageChange,
}) {
  const { colors, isDark } = useTheme()
  const { t } = useI18n()

  if (totalCount === 0) return null

  const PaginationButton = ({ onClick, disabled, children, title }) => (
    <button 
      onClick={onClick} 
      disabled={disabled} 
      title={title}
      className={`
        p-2.5 rounded-xl border transition-all
        ${isDark 
          ? 'border-white/10 hover:bg-white/10 disabled:hover:bg-transparent' 
          : 'border-gray-200 hover:bg-gray-100 disabled:hover:bg-transparent'
        }
        disabled:opacity-40 disabled:cursor-not-allowed
      `}
    >
      {children}
    </button>
  )

  return (
    <div className={`px-6 py-4 border-t ${colors.cardBorder} flex items-center justify-between animate-fade-in`}>
      {/* Left: Page Size Selector */}
      <div className={`flex items-center gap-3 text-sm ${colors.textMuted}`}>
        <span>{t('pagination.perPage')}</span>
        <select
          value={pageSize}
          onChange={(e) => onPageSizeChange(Number(e.target.value))}
          className={`
            px-3 py-2 rounded-xl border transition-all cursor-pointer
            ${isDark 
              ? 'bg-white/5 border-white/10 text-white' 
              : 'bg-white border-gray-200 text-gray-900'
            }
            focus:outline-none focus:ring-2 focus:ring-indigo-500/30
          `}
        >
          <option value={10}>10</option>
          <option value={20}>20</option>
          <option value={50}>50</option>
        </select>
        <span className={`px-3 py-1.5 rounded-lg ${isDark ? 'bg-white/5' : 'bg-gray-100'}`}>
          {t('pagination.totalItems', { count: totalCount })}
        </span>
      </div>

      {/* Right: Pagination Controls */}
      <div className="flex items-center gap-2">
        <PaginationButton 
          onClick={() => onPageChange(1)} 
          disabled={currentPage === 1} 
          title={t('pagination.first')}
        >
          <ChevronsLeft size={16} className={colors.textMuted} />
        </PaginationButton>
        
        <PaginationButton 
          onClick={() => onPageChange(currentPage - 1)} 
          disabled={currentPage === 1} 
          title={t('pagination.prev')}
        >
          <ChevronLeft size={16} className={colors.textMuted} />
        </PaginationButton>
        
        <div className={`
          px-5 py-2 rounded-xl font-medium text-sm
          ${isDark ? 'bg-indigo-500/20 text-indigo-300' : 'bg-indigo-100 text-indigo-600'}
        `}>
          {currentPage} / {totalPages}
        </div>
        
        <PaginationButton 
          onClick={() => onPageChange(currentPage + 1)} 
          disabled={currentPage === totalPages} 
          title={t('pagination.next')}
        >
          <ChevronRight size={16} className={colors.textMuted} />
        </PaginationButton>
        
        <PaginationButton 
          onClick={() => onPageChange(totalPages)} 
          disabled={currentPage === totalPages} 
          title={t('pagination.last')}
        >
          <ChevronsRight size={16} className={colors.textMuted} />
        </PaginationButton>
      </div>
    </div>
  )
}

export default AccountPagination
