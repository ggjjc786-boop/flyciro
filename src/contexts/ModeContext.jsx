import { createContext, useContext, useState, useEffect } from 'react'

const ModeContext = createContext()

export function ModeProvider({ children }) {
  const [mode, setMode] = useState(() => {
    const saved = localStorage.getItem('app_mode')
    return saved || 'kiro'
  })

  useEffect(() => {
    localStorage.setItem('app_mode', mode)
  }, [mode])

  const toggleMode = () => {
    setMode(prev => prev === 'kiro' ? 'cursor' : 'kiro')
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
