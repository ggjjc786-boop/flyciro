import { createContext, useContext, useState, ReactNode } from 'react'
import { X } from 'lucide-react'
import { useTheme } from './ThemeContext'

interface DialogContextType {
  showError: (title: string, message: string) => void
  showSuccess: (title: string, message: string) => void
  showConfirm: (title: string, message: string) => Promise<boolean>
}

const DialogContext = createContext<DialogContextType | undefined>(undefined)

interface DialogState {
  type: 'error' | 'success' | 'confirm'
  title: string
  message: string
  resolve?: (value: boolean) => void
}

export function DialogProvider({ children }: { children: ReactNode }) {
  const [dialog, setDialog] = useState<DialogState | null>(null)
  const { colors } = useTheme()

  const showError = (title: string, message: string) => {
    setDialog({ type: 'error', title, message })
  }

  const showSuccess = (title: string, message: string) => {
    setDialog({ type: 'success', title, message })
  }

  const showConfirm = (title: string, message: string): Promise<boolean> => {
    return new Promise((resolve) => {
      setDialog({ type: 'confirm', title, message, resolve })
    })
  }

  const handleClose = (confirmed: boolean = false) => {
    if (dialog?.resolve) {
      dialog.resolve(confirmed)
    }
    setDialog(null)
  }

  const value: DialogContextType = {
    showError,
    showSuccess,
    showConfirm,
  }

  return (
    <DialogContext.Provider value={value}>
      {children}
      
      {dialog && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 animate-fade-in">
          <div className={`${colors.card} rounded-2xl shadow-2xl max-w-md w-full mx-4 animate-scale-in`}>
            <div className={`px-6 py-4 border-b ${colors.cardBorder} flex items-center justify-between`}>
              <h3 className={`text-lg font-semibold ${colors.text}`}>{dialog.title}</h3>
              <button
                onClick={() => handleClose(false)}
                className={`p-1 rounded-lg ${colors.textMuted} hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors`}
              >
                <X size={20} />
              </button>
            </div>
            
            <div className="px-6 py-4">
              <p className={`${colors.text} whitespace-pre-wrap`}>{dialog.message}</p>
            </div>
            
            <div className="px-6 py-4 flex gap-3 justify-end">
              {dialog.type === 'confirm' ? (
                <>
                  <button
                    onClick={() => handleClose(false)}
                    className={`px-4 py-2 rounded-lg ${colors.btnSecondary} border transition-colors`}
                  >
                    取消
                  </button>
                  <button
                    onClick={() => handleClose(true)}
                    className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
                  >
                    确认
                  </button>
                </>
              ) : (
                <button
                  onClick={() => handleClose(false)}
                  className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
                >
                  确定
                </button>
              )}
            </div>
          </div>
        </div>
      )}
    </DialogContext.Provider>
  )
}

export function useDialog() {
  const context = useContext(DialogContext)
  if (!context) throw new Error('useDialog must be used within DialogProvider')
  return context
}
