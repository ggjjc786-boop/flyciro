import { message as tauriMessage, ask as tauriAsk, confirm as tauriConfirm } from '@tauri-apps/plugin-dialog';

/**
 * 显示信息提示
 */
export async function showMessage(message: string, title: string = '提示'): Promise<void> {
  await tauriMessage(message, { title, kind: 'info' });
}

/**
 * 显示成功提示
 */
export async function showSuccess(message: string, title: string = '成功'): Promise<void> {
  await tauriMessage(message, { title, kind: 'info' });
}

/**
 * 显示错误提示
 */
export async function showError(message: string, title: string = '错误'): Promise<void> {
  await tauriMessage(message, { title, kind: 'error' });
}

/**
 * 显示警告提示
 */
export async function showWarning(message: string, title: string = '警告'): Promise<void> {
  await tauriMessage(message, { title, kind: 'warning' });
}

/**
 * 显示确认对话框
 */
export async function showConfirm(message: string, title: string = '确认'): Promise<boolean> {
  return await tauriConfirm(message, { title, kind: 'warning' });
}

/**
 * 显示询问对话框（是/否）
 */
export async function showAsk(message: string, title: string = '询问'): Promise<boolean> {
  return await tauriAsk(message, { title, kind: 'info' });
}
