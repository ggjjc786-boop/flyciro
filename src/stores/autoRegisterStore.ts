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

interface AutoRegisterState {
  accounts: Account[];
  settings: Settings;
  isLoading: boolean;

  setAccounts: (accounts: Account[]) => void;
  setSettings: (settings: Settings) => void;
  setIsLoading: (isLoading: boolean) => void;
}

export const useStore = create<AutoRegisterState>((set) => ({
  accounts: [],
  settings: {
    browser_mode: 'background',
  },
  isLoading: false,

  setAccounts: (accounts) => set({ accounts }),
  setSettings: (settings) => set({ settings }),
  setIsLoading: (isLoading) => set({ isLoading }),
}));
