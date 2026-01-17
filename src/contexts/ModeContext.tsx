import { createContext, useContext, useState, useEffect, ReactNode } from 'react'

type AppMode = 'kiro' | 'aws'

interface ModeContextType {
  mode: AppMode
  setMode: (mode: AppMode) => void
  toggleMode: () => void
}

const ModeContext = createContext<ModeContextType | undefined>(undefined)

export function ModeProvider({ children }: { children: ReactNode }) {
  const [mode, setMode] = useState<AppMode>(() => {
    const saved = localStorage.getItem('app_mode')
    return (saved as AppMode) || 'kiro'
  })

  useEffect(() => {
    localStorage.setItem('app_mode', mode)
  }, [mode])

  const toggleMode = () => {
    setMode(prev => prev === 'kiro' ? 'aws' : 'kiro')
  }

  return (
    <ModeContext.Provider value={{ mode, setMode, toggleMode }}>
      {children}
    </ModeContext.Provider>
  )
}

export function useMode() {
  const context = useContext(ModeContext)
  if (!context) {
    throw new Error('useMode must be used within a ModeProvider')
  }
  return context
}
