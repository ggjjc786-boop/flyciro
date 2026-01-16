import { create } from 'zustand';

export interface Account {
  id: number;
  email: string;
  email_password: string;
  client_id: string;
  refresh_token: string;
  kiro_password: string | null;
  status: 'not_registered' | 'in_progress' | 'registered' | 'error';
  error_reason: string | null;
  created_at: string;
  updated_at: string;
}

export interface Settings {
  browser_mode: 'background' | 'foreground';
}

interface AppState {
  theme: 'light' | 'dark';
  accounts: Account[];
  settings: Settings;
  isLoading: boolean;
  titleBarVisible: boolean;

  setTheme: (theme: 'light' | 'dark') => void;
  setAccounts: (accounts: Account[]) => void;
  setSettings: (settings: Settings) => void;
  setIsLoading: (isLoading: boolean) => void;
  setTitleBarVisible: (visible: boolean) => void;
}

export const useStore = create<AppState>((set) => ({
  theme: (localStorage.getItem('theme') as 'light' | 'dark') || 'light',
  accounts: [],
  settings: {
    browser_mode: 'background',
  },
  isLoading: false,
  titleBarVisible: true,

  setTheme: (theme) => {
    localStorage.setItem('theme', theme);
    document.documentElement.setAttribute('data-theme', theme);
    set({ theme });
  },

  setAccounts: (accounts) => set({ accounts }),

  setSettings: (settings) => set({ settings }),

  setIsLoading: (isLoading) => set({ isLoading }),

  setTitleBarVisible: (visible) => set({ titleBarVisible: visible }),
}));
