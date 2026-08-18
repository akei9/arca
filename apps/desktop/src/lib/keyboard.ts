export function isMacPlatform(): boolean {
  if (typeof navigator === 'undefined') {
    return false;
  }

  return navigator.platform.includes('Mac') || navigator.userAgent.includes('Macintosh');
}

export function primaryModifierLabel(): string {
  return isMacPlatform() ? '⌘' : 'Ctrl';
}

export function primaryModifierPressed(event: KeyboardEvent): boolean {
  return isMacPlatform() ? event.metaKey : event.ctrlKey;
}

export function shortcutLabel(...parts: string[]): string {
  return parts.join(' + ');
}

export function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  return (
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement ||
    target.isContentEditable
  );
}
