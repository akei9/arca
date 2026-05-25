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
const THEME_STORAGE_KEY = 'arca.theme';

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

export function coerceThemeName(value: string | null | undefined): ThemeName {
  return value === 'ink' ? 'ink' : 'paper';
}

export function loadThemePreference() {
  if (typeof localStorage === 'undefined') {
    return;
  }

  try {
    uiState.theme = coerceThemeName(localStorage.getItem(THEME_STORAGE_KEY));
  } catch {
    uiState.theme = 'paper';
  }
}

export function setThemePreference(theme: ThemeName) {
  uiState.theme = theme;

  if (typeof localStorage === 'undefined') {
    return;
  }

  try {
    localStorage.setItem(THEME_STORAGE_KEY, theme);
  } catch {
    // Theme persistence is best-effort; the active in-memory theme still applies.
  }
}
