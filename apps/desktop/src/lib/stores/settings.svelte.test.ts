import { describe, expect, it } from 'vitest';
import { normalizeSettings, uiThemeFor } from './settings.svelte';

describe('normalizeSettings', () => {
  it('preserves disabled timers and canonicalizes legacy themes', () => {
    expect(
      normalizeSettings({
        autoLockTimeoutMinutes: null,
        clipboardClearSeconds: null,
        theme: 'amber',
        fontSize: 14,
      }),
    ).toEqual({
      autoLockTimeoutMinutes: null,
      clipboardClearSeconds: null,
      theme: 'ink',
      fontSize: 14,
    });
  });

  it('falls back invalid timer values and clamps font size', () => {
    expect(
      normalizeSettings({
        autoLockTimeoutMinutes: 0,
        clipboardClearSeconds: 17,
        theme: 'terminal',
        fontSize: 100,
      }),
    ).toEqual({
      autoLockTimeoutMinutes: 15,
      clipboardClearSeconds: 30,
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
