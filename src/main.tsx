import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from '@tauri-apps/api/window'
import App from "./App";
import { ThemeProvider } from './contexts/ThemeContext';
import { ModeProvider } from './contexts/ModeContext';
import { DialogProvider } from './contexts/DialogContext';
import { I18nProvider } from './i18n';
import "./index.css";

// 生产环境禁用浏览器快捷键
if (import.meta.env.PROD) {
  document.addEventListener('keydown', (e) => {
    if (e.key === 'F5' || e.key === 'F12') {
      e.preventDefault();
      return false;
    }
    
    if (e.ctrlKey) {
      const key = e.key.toLowerCase();
      if (['r', 'u', 'p', 's', 'g', 'f'].includes(key)) {
        e.preventDefault();
        return false;
      }
      if (e.shiftKey && ['i', 'j'].includes(key)) {
        e.preventDefault();
        return false;
      }
    }
  });
  
  document.addEventListener('contextmenu', (e) => {
    e.preventDefault();
    return false;
  });
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <I18nProvider>
      <ThemeProvider>
        <ModeProvider>
          <DialogProvider>
            <App />
          </DialogProvider>
        </ModeProvider>
      </ThemeProvider>
    </I18nProvider>
  </React.StrictMode>,
);

// 页面加载完成后显示窗口
document.addEventListener('DOMContentLoaded', () => {
  setTimeout(() => {
    getCurrentWindow().show();
  }, 100);
});
