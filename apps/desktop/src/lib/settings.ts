import type { Settings } from './ipc';

export const SETTINGS_LIMITS = {
  autoLockTimeoutMinutes: {
    defaultValue: 15,
    min: 1,
    max: 240,
    step: 1,
  },
  clipboardClearSeconds: {
    defaultValue: 30,
    min: 5,
    max: 300,
    step: 5,
  },
  entryRevisionLimit: {
    defaultValue: 5,
    min: 0,
    max: 25,
    step: 1,
  },
  fontSize: {
    defaultValue: 13,
    min: 11,
    max: 16,
    step: 1,
  },
} as const;

export const RELEASE_THEME_OPTIONS = [
  { value: 'paper', label: 'paper' },
  { value: 'ink', label: 'ink' },
] as const;

export const RELEASE_AUTO_LOCK_OPTIONS = [
  { value: '1', label: '1 min' },
  { value: '5', label: '5 min' },
  { value: '15', label: '15 min' },
  { value: '60', label: '60 min' },
  { value: 'never', label: 'never' },
] as const;

export const RELEASE_CLIPBOARD_CLEAR_OPTIONS = [
  { value: '15', label: '15s' },
  { value: '30', label: '30s' },
  { value: '60', label: '60s' },
  { value: '120', label: '120s' },
] as const;

export const RELEASE_ENTRY_REVISION_OPTIONS = [
  { value: '0', label: 'off' },
  { value: '1', label: '1' },
  { value: '3', label: '3' },
  { value: '5', label: '5' },
  { value: '10', label: '10' },
  { value: '25', label: '25' },
] as const;

export const DEFAULT_SETTINGS: Settings = {
  autoLockTimeoutMinutes: SETTINGS_LIMITS.autoLockTimeoutMinutes.defaultValue,
  clipboardClearSeconds: SETTINGS_LIMITS.clipboardClearSeconds.defaultValue,
  entryRevisionLimit: SETTINGS_LIMITS.entryRevisionLimit.defaultValue,
  theme: 'paper',
  fontSize: SETTINGS_LIMITS.fontSize.defaultValue,
};

export function normalizeSettings(settings: Settings): Settings {
  return {
    autoLockTimeoutMinutes: normalizeOptionalInteger(
      settings.autoLockTimeoutMinutes,
      SETTINGS_LIMITS.autoLockTimeoutMinutes.defaultValue,
      SETTINGS_LIMITS.autoLockTimeoutMinutes.min,
      SETTINGS_LIMITS.autoLockTimeoutMinutes.max,
      SETTINGS_LIMITS.autoLockTimeoutMinutes.step,
    ),
    clipboardClearSeconds: normalizeOptionalInteger(
      settings.clipboardClearSeconds,
      SETTINGS_LIMITS.clipboardClearSeconds.defaultValue,
      SETTINGS_LIMITS.clipboardClearSeconds.min,
      SETTINGS_LIMITS.clipboardClearSeconds.max,
      SETTINGS_LIMITS.clipboardClearSeconds.step,
    ),
    entryRevisionLimit: normalizeInteger(
      settings.entryRevisionLimit,
      SETTINGS_LIMITS.entryRevisionLimit.defaultValue,
      SETTINGS_LIMITS.entryRevisionLimit.min,
      SETTINGS_LIMITS.entryRevisionLimit.max,
      SETTINGS_LIMITS.entryRevisionLimit.step,
    ),
    theme: themeForUi(uiThemeFor(settings.theme)),
    fontSize: normalizeFontSize(settings.fontSize),
  };
}

export function themeForUi(theme: 'paper' | 'ink'): Settings['theme'] {
  return theme;
}

export function uiThemeFor(theme: Settings['theme']): 'paper' | 'ink' {
  return theme === 'ink' || theme === 'amber' ? 'ink' : 'paper';
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

function normalizeInteger(
  value: number | null | undefined,
  fallback: number,
  min: number,
  max: number,
  step = 1,
): number {
  if (value === null || value === undefined || !Number.isFinite(value)) {
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
    return SETTINGS_LIMITS.fontSize.defaultValue;
  }

  return Math.min(
    SETTINGS_LIMITS.fontSize.max,
    Math.max(SETTINGS_LIMITS.fontSize.min, Math.trunc(value)),
  );
}
