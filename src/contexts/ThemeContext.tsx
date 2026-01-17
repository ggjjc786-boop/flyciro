import { createContext, useContext, useState, useEffect, ReactNode } from 'react'

interface ThemeContextType {
  theme: string
  setTheme: (theme: string) => void
  colors: Record<string, string>
  isDark: boolean
  themes: Record<string, { name: string; class: string }>
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined)

export const themes = {
  dark: {
    name: '深色',
    class: 'theme-dark',
  },
  light: {
    name: '浅色',
    class: 'theme-light',
  },
  aurora: {
    name: '极光',
    class: 'theme-aurora',
  }
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setTheme] = useState(() => {
    return localStorage.getItem('app-theme') || 'dark'
  })

  useEffect(() => {
    localStorage.setItem('app-theme', theme)
    document.documentElement.className = themes[theme as keyof typeof themes]?.class || ''
  }, [theme])

  const isDark = theme === 'dark' || theme === 'aurora'

  const colors = {
    // 主背景
    main: 'bg-[#0c0c14]',
    mainGradient: 'bg-[#0c0c14]',
    
    // 侧边栏
    sidebar: 'bg-[#0a0a12]/95 backdrop-blur-xl border-r border-white/5',
    sidebarText: 'text-slate-400',
    sidebarHover: 'hover:bg-white/5 hover:text-white',
    sidebarActive: 'bg-gradient-to-r from-indigo-500/20 to-purple-500/10 text-white',
    sidebarMuted: 'text-slate-500',
    sidebarCard: 'bg-white/5',
    
    // 卡片
    card: 'glass-card rounded-2xl',
    cardBorder: 'border-white/5',
    
    // 文字
    text: 'text-slate-100',
    textSecondary: 'text-slate-300',
    textMuted: 'text-slate-500',
    
    // 输入框
    input: 'input-aurora',
    inputFocus: 'focus:border-indigo-500',
    
    // 按钮
    btnPrimary: 'btn-aurora',
    btnGhost: 'btn-ghost',
    btnDanger: 'bg-red-500/20 text-red-400 hover:bg-red-500/30 border border-red-500/30',
    
    // 分割线
    divider: 'bg-white/10',
    
    // 表格
    tableHeader: 'text-slate-500',
    tableRow: 'border-white/5 hover:bg-white/5',
  }

  return (
    <ThemeContext.Provider value={{ theme, setTheme, colors, isDark, themes }}>
      {children}
    </ThemeContext.Provider>
  )
}

export function useTheme() {
  const context = useContext(ThemeContext)
  if (!context) {
    throw new Error('useTheme must be used within ThemeProvider')
  }
  return context
}
