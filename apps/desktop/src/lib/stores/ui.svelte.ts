export type ViewName = 'unlock' | 'list' | 'detail' | 'edit' | 'settings' | 'generator';
export type ThemeName = 'terminal' | 'amber';

export interface Notification {
  kind: 'info' | 'success' | 'warning' | 'error';
  message: string;
}

export const uiState = $state({
  theme: 'terminal' as ThemeName,
  view: 'unlock' as ViewName,
  clipboardTimer: null as ReturnType<typeof setTimeout> | null,
  notification: null as Notification | null,
});
