import { createContext, useContext, useState, useCallback, ReactNode } from 'react'

interface DialogContextType {
  showConfirm: (title: string, message: string, options?: ConfirmOptions) => Promise<boolean>
  showSuccess: (title: string, message: string) => Promise<void>
  showError: (title: string, message: string) => Promise<void>
  showInfo: (title: string, message: string) => Promise<void>
}

interface ConfirmOptions {
  confirmText?: string
  cancelText?: string
}

interface DialogState {
  type: 'confirm' | 'success' | 'error' | 'info'
  title: string
  message: string
  confirmText?: string
  cancelText?: string
}

const DialogContext = createContext<DialogContextType | null>(null)

/**
 * 简单的确认对话框组件
 */
function ConfirmDialog({
  type,
  title,
  message,
  confirmText = '确定',
  cancelText = '取消',
  onConfirm,
  onCancel,
}: {
  type: string
  title: string
  message: string
  confirmText?: string
  cancelText?: string
  onConfirm: () => void
  onCancel: () => void
}) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="glass-card p-6 max-w-md w-full mx-4 animate-scale-in">
        <h3 className="text-lg font-semibold text-white mb-2">{title}</h3>
        <p className="text-slate-300 mb-6">{message}</p>
        <div className="flex gap-3 justify-end">
          {type === 'confirm' && (
            <button onClick={onCancel} className="btn-ghost px-4 py-2">
              {cancelText}
            </button>
          )}
          <button onClick={onConfirm} className="btn-aurora px-4 py-2">
            {confirmText}
          </button>
        </div>
      </div>
    </div>
  )
}

/**
 * 全局弹窗 Provider
 */
export function DialogProvider({ children }: { children: ReactNode }) {
  const [dialog, setDialog] = useState<DialogState | null>(null)
  const [resolveRef, setResolveRef] = useState<((value: boolean) => void) | null>(null)

  // 显示确认弹窗，返回 Promise<boolean>
  const showConfirm = useCallback((title: string, message: string, options: ConfirmOptions = {}) => {
    return new Promise<boolean>((resolve) => {
      setResolveRef(() => resolve)
      setDialog({
        type: 'confirm',
        title,
        message,
        confirmText: options.confirmText,
        cancelText: options.cancelText,
      })
    })
  }, [])

  // 显示成功弹窗
  const showSuccess = useCallback((title: string, message: string) => {
    return new Promise<void>((resolve) => {
      setResolveRef(() => () => resolve())
      setDialog({
        type: 'success',
        title,
        message,
      })
    })
  }, [])

  // 显示错误弹窗
  const showError = useCallback((title: string, message: string) => {
    return new Promise<void>((resolve) => {
      setResolveRef(() => () => resolve())
      setDialog({
        type: 'error',
        title,
        message,
      })
    })
  }, [])

  // 显示信息弹窗
  const showInfo = useCallback((title: string, message: string) => {
    return new Promise<void>((resolve) => {
      setResolveRef(() => () => resolve())
      setDialog({
        type: 'info',
        title,
        message,
      })
    })
  }, [])

  const handleConfirm = useCallback(() => {
    if (resolveRef) resolveRef(true)
    setDialog(null)
    setResolveRef(null)
  }, [resolveRef])

  const handleCancel = useCallback(() => {
    if (resolveRef) resolveRef(false)
    setDialog(null)
    setResolveRef(null)
  }, [resolveRef])

  return (
    <DialogContext.Provider value={{ showConfirm, showSuccess, showError, showInfo }}>
      {children}
      {dialog && (
        <ConfirmDialog
          type={dialog.type}
          title={dialog.title}
          message={dialog.message}
          confirmText={dialog.confirmText}
          cancelText={dialog.cancelText}
          onConfirm={handleConfirm}
          onCancel={handleCancel}
        />
      )}
    </DialogContext.Provider>
  )
}

/**
 * 使用全局弹窗 Hook
 */
export function useDialog() {
  const context = useContext(DialogContext)
  if (!context) {
    throw new Error('useDialog must be used within a DialogProvider')
  }
  return context
}
