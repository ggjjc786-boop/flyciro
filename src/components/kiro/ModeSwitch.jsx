import { Sparkles, MousePointer2 } from 'lucide-react'
import { useMode } from '../contexts/ModeContext'

function ModeSwitch() {
  const { mode, toggleMode } = useMode()

  return (
    <button
      onClick={toggleMode}
      className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 border border-white/10 transition-all group"
      title={`切换到 ${mode === 'kiro' ? 'Cursor' : 'Kiro'} 模式`}
    >
      <div className="relative flex items-center">
        {/* Kiro 图标 */}
        <div className={`w-6 h-6 rounded-md flex items-center justify-center transition-all ${
          mode === 'kiro' 
            ? 'bg-gradient-to-br from-indigo-500 to-purple-600 shadow-lg shadow-indigo-500/30' 
            : 'bg-white/5'
        }`}>
          <Sparkles size={14} className={mode === 'kiro' ? 'text-white' : 'text-slate-500'} />
        </div>
        
        {/* 切换指示器 */}
        <div className="w-8 h-5 mx-1.5 rounded-full bg-white/10 relative">
          <div className={`absolute top-0.5 w-4 h-4 rounded-full transition-all duration-300 ${
            mode === 'kiro' 
              ? 'left-0.5 bg-gradient-to-br from-indigo-500 to-purple-600' 
              : 'left-3 bg-gradient-to-br from-cyan-500 to-blue-600'
          }`} />
        </div>
        
        {/* Cursor 图标 */}
        <div className={`w-6 h-6 rounded-md flex items-center justify-center transition-all ${
          mode === 'cursor' 
            ? 'bg-gradient-to-br from-cyan-500 to-blue-600 shadow-lg shadow-cyan-500/30' 
            : 'bg-white/5'
        }`}>
          <MousePointer2 size={14} className={mode === 'cursor' ? 'text-white' : 'text-slate-500'} />
        </div>
      </div>
      
      <span className={`text-xs font-medium transition-colors ${
        mode === 'kiro' ? 'text-indigo-400' : 'text-cyan-400'
      }`}>
        {mode === 'kiro' ? 'Kiro' : 'Cursor'}
      </span>
    </button>
  )
}

export default ModeSwitch
