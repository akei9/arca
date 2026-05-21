export type ShortcutHandler = (event: KeyboardEvent) => void;

const handlers = new Map<string, ShortcutHandler>();

export function registerShortcut(key: string, handler: ShortcutHandler): () => void {
  handlers.set(key.toLowerCase(), handler);
  return () => {
    handlers.delete(key.toLowerCase());
  };
}

export function handleShortcut(event: KeyboardEvent): void {
  const handler = handlers.get(event.key.toLowerCase());

  if (handler) {
    handler(event);
  }
}
