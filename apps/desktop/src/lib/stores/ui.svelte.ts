export type ViewName =
  | 'unlock'
  | 'list'
  | 'detail'
  | 'edit'
  | 'settings'
  | 'generator'
  | 'audit'
  | 'shared';
export type ThemeName = 'paper' | 'ink';
export type UnlockSurface = 'two-pane' | 'sealed';

export interface Notification {
  kind: 'info' | 'success' | 'warning' | 'error';
  message: string;
}

export const uiState = $state({
  theme: 'paper' as ThemeName,
  view: 'unlock' as ViewName,
  unlockSurface: 'two-pane' as UnlockSurface,
  sealedPromptOpen: false,
  clipboardTimer: null as ReturnType<typeof setTimeout> | null,
  notification: null as Notification | null,
});
