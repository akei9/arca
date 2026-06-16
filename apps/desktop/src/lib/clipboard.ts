import { runtimeSettings } from './stores/settings.svelte';

let clearTimer: ReturnType<typeof setTimeout> | null = null;

export async function writeConfiguredClipboardText(value: string): Promise<boolean> {
  return writeClipboardText(value, runtimeSettings.current.clipboardClearSeconds);
}

export async function writeClipboardText(
  value: string,
  clearAfterSeconds: number | null | undefined = null,
): Promise<boolean> {
  if (!value) {
    return false;
  }

  let copied = false;

  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(value);
      copied = true;
    }
  } catch {
    // Fall through to the textarea fallback for webviews without clipboard grants.
  }

  if (!copied) {
    copied = copyWithTextarea(value);
  }

  if (copied) {
    scheduleClipboardClear(clearAfterSeconds);
  }

  return copied;
}

export function cancelClipboardClear() {
  if (clearTimer) {
    clearTimeout(clearTimer);
    clearTimer = null;
  }
}

function copyWithTextarea(value: string): boolean {
  try {
    const activeElement = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const textarea = document.createElement('textarea');

    textarea.value = value;
    textarea.setAttribute('readonly', '');
    textarea.style.position = 'fixed';
    textarea.style.opacity = '0';
    textarea.style.pointerEvents = 'none';
    document.body.append(textarea);
    textarea.select();

    const copied = document.execCommand('copy');
    textarea.remove();
    activeElement?.focus();

    return copied;
  } catch {
    return false;
  }
}

function scheduleClipboardClear(clearAfterSeconds: number | null | undefined) {
  cancelClipboardClear();

  if (clearAfterSeconds === null || clearAfterSeconds === undefined) {
    return;
  }

  const delayMs = Math.max(0, clearAfterSeconds) * 1000;

  clearTimer = setTimeout(() => {
    clearTimer = null;
    void clearClipboardText();
  }, delayMs);
}

async function clearClipboardText(): Promise<void> {
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText('');
      return;
    }
  } catch {
    // Fall through to the textarea fallback for webviews without clipboard grants.
  }

  copyWithTextarea('');
}
