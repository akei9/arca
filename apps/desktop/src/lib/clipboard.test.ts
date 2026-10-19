import { isTauri } from '@tauri-apps/api/core';
import {
  clear as clearNativeClipboard,
  writeSecret as writeNativeClipboardSecret,
} from '@tauri-apps/plugin-clipboard-manager';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({
  clear: vi.fn(),
  writeSecret: vi.fn(),
}));

import { cancelClipboardClear, writeClipboardText } from './clipboard';

const testValue = 'clipboard-test-value';
const writeBrowserClipboardText = vi.fn(async (_value: string) => undefined);

describe('clipboard writes', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.mocked(isTauri).mockReset().mockReturnValue(false);
    vi.mocked(clearNativeClipboard).mockReset().mockResolvedValue(undefined);
    vi.mocked(writeNativeClipboardSecret).mockReset().mockResolvedValue(undefined);
    writeBrowserClipboardText.mockClear();
    vi.stubGlobal('navigator', {
      clipboard: {
        writeText: writeBrowserClipboardText,
      },
    });
  });

  afterEach(() => {
    cancelClipboardClear();
    vi.unstubAllGlobals();
    vi.useRealTimers();
  });

  it('marks native clipboard values as sensitive', async () => {
    await expect(writeClipboardText(testValue)).resolves.toBe(true);

    expect(writeNativeClipboardSecret).toHaveBeenCalledOnce();
    expect(writeNativeClipboardSecret).toHaveBeenCalledWith(testValue);
    expect(writeBrowserClipboardText).not.toHaveBeenCalled();
  });

  it('falls back to the browser clipboard when the native write fails', async () => {
    vi.mocked(writeNativeClipboardSecret).mockRejectedValue(new Error('native clipboard unavailable'));

    await expect(writeClipboardText(testValue)).resolves.toBe(true);

    expect(writeBrowserClipboardText).toHaveBeenCalledOnce();
    expect(writeBrowserClipboardText).toHaveBeenCalledWith(testValue);
  });

  it('does not downgrade a failed native secret write inside Tauri', async () => {
    vi.mocked(isTauri).mockReturnValue(true);
    vi.mocked(writeNativeClipboardSecret).mockRejectedValue(new Error('native clipboard unavailable'));

    await expect(writeClipboardText(testValue)).resolves.toBe(false);

    expect(writeBrowserClipboardText).not.toHaveBeenCalled();
  });

  it('does not invoke a clipboard API for an empty value', async () => {
    await expect(writeClipboardText('')).resolves.toBe(false);

    expect(writeNativeClipboardSecret).not.toHaveBeenCalled();
    expect(writeBrowserClipboardText).not.toHaveBeenCalled();
  });

  it('retains timed clipboard clearing as defense in depth', async () => {
    await expect(writeClipboardText(testValue, 5)).resolves.toBe(true);

    await vi.advanceTimersByTimeAsync(4_999);
    expect(clearNativeClipboard).not.toHaveBeenCalled();

    await vi.advanceTimersByTimeAsync(1);
    expect(clearNativeClipboard).toHaveBeenCalledOnce();
  });
});
