export type ViewName = 'unlock' | 'list' | 'detail' | 'edit' | 'settings' | 'generator';
export type ThemeName = 'paper' | 'ink';

export interface Notification {
  kind: 'info' | 'success' | 'warning' | 'error';
  message: string;
}

export const uiState = $state({
  theme: 'paper' as ThemeName,
  view: 'unlock' as ViewName,
  clipboardTimer: null as ReturnType<typeof setTimeout> | null,
  notification: null as Notification | null,
});
