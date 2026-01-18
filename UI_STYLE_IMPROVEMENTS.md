# UI 样式优化说明

## 优化内容

### 1. 主容器样式
- 使用 Tailwind CSS 替代自定义 CSS
- 集成主题颜色系统 (`useTheme` hook)
- 响应式布局优化

### 2. 组件样式统一
所有 AutoRegister 组件现在使用：
- 主项目的 `colors` 对象进行主题适配
- Tailwind 工具类进行布局
- 统一的卡片样式 (`card-glow`, `rounded-2xl`, `shadow-sm`)
- 统一的边框和间距

### 3. 需要进一步优化的组件

#### ImportPanel.tsx
- 需要添加 `useTheme` hook
- 更新样式使用 Tailwind + colors

#### ControlPanel.tsx  
- 需要添加 `useTheme` hook
- 更新按钮样式
- 优化模态框样式

#### AccountsTable.tsx
- 需要添加 `useTheme` hook
- 更新表格样式
- 优化分页控件

### 4. 建议的样式模式

```tsx
import { useTheme } from '../../contexts/ThemeContext';

export function Component() {
  const { colors, isDark } = useTheme();
  
  return (
    <div className={`${colors.card} rounded-2xl border ${colors.cardBorder} p-6`}>
      <h2 className={`text-lg font-semibold ${colors.text} mb-4`}>标题</h2>
      <p className={colors.textMuted}>描述文本</p>
      
      <button className={`px-4 py-2 rounded-lg ${isDark ? 'bg-blue-500/20 hover:bg-blue-500/30' : 'bg-blue-100 hover:bg-blue-200'} transition-colors`}>
        按钮
      </button>
    </div>
  );
}
```

### 5. 颜色系统

主项目使用的 colors 对象包含：
- `colors.main` - 主背景色
- `colors.card` - 卡片背景色
- `colors.cardBorder` - 卡片边框色
- `colors.text` - 主文本色
- `colors.textMuted` - 次要文本色
- `colors.textSecondary` - 辅助文本色

### 6. 下一步

1. 更新所有 AutoRegister 子组件使用 `useTheme`
2. 移除所有自定义 CSS 文件
3. 使用 Tailwind 重写所有样式
4. 确保暗色/亮色主题切换正常工作
