import { getSettings, updateSettings, type Settings } from '../ipc';
import { setThemePreference, type ThemeName } from './ui.svelte';

const DEFAULT_AUTO_LOCK_TIMEOUT_MINUTES = 15;
const DEFAULT_CLIPBOARD_CLEAR_SECONDS = 30;
const DEFAULT_FONT_SIZE = 13;

export const DEFAULT_SETTINGS: Settings = {
  autoLockTimeoutMinutes: DEFAULT_AUTO_LOCK_TIMEOUT_MINUTES,
  clipboardClearSeconds: DEFAULT_CLIPBOARD_CLEAR_SECONDS,
  theme: 'paper',
  fontSize: DEFAULT_FONT_SIZE,
};

export const runtimeSettings = $state({
  current: { ...DEFAULT_SETTINGS },
  loaded: false,
});

export async function loadRuntimeSettings(): Promise<Settings> {
  const settings = normalizeSettings(await getSettings());
  applyRuntimeSettings(settings);

  return settings;
}

export async function saveRuntimeSettings(settings: Settings): Promise<Settings> {
  const normalized = normalizeSettings(settings);
  await updateSettings(normalized);
  applyRuntimeSettings(normalized);

  return normalized;
}

export function applyRuntimeSettings(settings: Settings) {
  runtimeSettings.current = settings;
  runtimeSettings.loaded = true;
  setThemePreference(uiThemeFor(runtimeSettings.current.theme));
}

export function themeForUi(theme: ThemeName): Settings['theme'] {
  return theme;
}

export function uiThemeFor(theme: Settings['theme']): ThemeName {
  return theme === 'ink' || theme === 'amber' ? 'ink' : 'paper';
}

function normalizeSettings(settings: Settings): Settings {
  return {
    autoLockTimeoutMinutes: normalizeOptionalInteger(
      settings.autoLockTimeoutMinutes,
      DEFAULT_AUTO_LOCK_TIMEOUT_MINUTES,
      1,
      240,
    ),
    clipboardClearSeconds: normalizeOptionalInteger(
      settings.clipboardClearSeconds,
      DEFAULT_CLIPBOARD_CLEAR_SECONDS,
      5,
      300,
      5,
    ),
    theme: themeForUi(uiThemeFor(settings.theme)),
    fontSize: normalizeFontSize(settings.fontSize),
  };
}

function normalizeOptionalInteger(
  value: number | null | undefined,
  fallback: number | null,
  min: number,
  max: number,
  step = 1,
): number | null {
  if (value === null) {
    return null;
  }

  if (value === undefined || !Number.isFinite(value)) {
    return fallback;
  }

  const integer = Math.trunc(value);

  if (integer < min || integer > max || integer % step !== 0) {
    return fallback;
  }

  return integer;
}

function normalizeFontSize(value: number | null | undefined): number {
  if (value === null || value === undefined || !Number.isFinite(value)) {
    return DEFAULT_FONT_SIZE;
  }

  return Math.min(16, Math.max(11, Math.trunc(value)));
}
