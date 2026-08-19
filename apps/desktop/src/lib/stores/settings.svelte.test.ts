import { describe, expect, it } from 'vitest';
import {
  RELEASE_AUTO_LOCK_OPTIONS,
  RELEASE_CLIPBOARD_CLEAR_OPTIONS,
  RELEASE_ENTRY_REVISION_OPTIONS,
  RELEASE_THEME_OPTIONS,
  SETTINGS_LIMITS,
  normalizeSettings,
  uiThemeFor,
} from '../settings';

describe('normalizeSettings', () => {
  it('preserves disabled timers and canonicalizes legacy themes', () => {
    expect(
      normalizeSettings({
        autoLockTimeoutMinutes: null,
        clipboardClearSeconds: null,
        entryRevisionLimit: 10,
        theme: 'amber',
        fontSize: 14,
      }),
    ).toEqual({
      autoLockTimeoutMinutes: null,
      clipboardClearSeconds: null,
      entryRevisionLimit: 10,
      theme: 'ink',
      fontSize: 14,
    });
  });

  it('falls back invalid timer values and clamps font size', () => {
    expect(
      normalizeSettings({
        autoLockTimeoutMinutes: 0,
        clipboardClearSeconds: 17,
        entryRevisionLimit: 100,
        theme: 'terminal',
        fontSize: 100,
      }),
    ).toEqual({
      autoLockTimeoutMinutes: 15,
      clipboardClearSeconds: 30,
      entryRevisionLimit: 5,
      theme: 'paper',
      fontSize: 16,
    });
  });
});

describe('uiThemeFor', () => {
  it('maps supported and legacy themes to UI themes', () => {
    expect(uiThemeFor('paper')).toBe('paper');
    expect(uiThemeFor('ink')).toBe('ink');
    expect(uiThemeFor('amber')).toBe('ink');
    expect(uiThemeFor('terminal')).toBe('paper');
  });
});

describe('release settings surface', () => {
  it('exposes only settings that are enforced at runtime for the first desktop release', () => {
    expect(RELEASE_THEME_OPTIONS.map((option) => option.value)).toEqual(['paper', 'ink']);
    expect(RELEASE_AUTO_LOCK_OPTIONS.map((option) => option.value)).toEqual(['1', '5', '15', '60', 'never']);
    expect(RELEASE_CLIPBOARD_CLEAR_OPTIONS.map((option) => option.value)).toEqual(['15', '30', '60', '120']);
    expect(RELEASE_ENTRY_REVISION_OPTIONS.map((option) => option.value)).toEqual(['0', '1', '3', '5', '10', '25']);
    expect(SETTINGS_LIMITS.entryRevisionLimit).toEqual({
      defaultValue: 5,
      min: 0,
      max: 25,
      step: 1,
    });
    expect(SETTINGS_LIMITS.fontSize).toEqual({
      defaultValue: 13,
      min: 11,
      max: 16,
      step: 1,
    });
  });
});
