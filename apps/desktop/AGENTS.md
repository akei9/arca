# Desktop Agent Guide

This package contains the Svelte 5 frontend and the Tauri v2 application shell.

## Commands

- `pnpm --filter @arca/desktop dev` - start the Vite dev server.
- `pnpm --filter @arca/desktop build` - build the frontend.
- `pnpm --filter @arca/desktop tauri` - run Tauri CLI commands.

## Frontend Rules

- Do not log vault entries, generated passwords, master passwords, clipboard values, or IPC payloads containing secrets.
- Keep plaintext password display explicit and user-driven. Mask secrets in list views and passive UI.
- Use typed wrappers in `src/lib/ipc.ts` for Tauri commands instead of calling `invoke` directly from components.
- Keep state in existing Svelte stores under `src/lib/stores` when it is shared across views.
- Match the existing terminal-inspired visual system in `src/app.css` and component primitives.

## Tauri Rules

- Keep Tauri commands thin and delegate vault logic to `vault-core`.
- Use `zeroize::Zeroizing` or equivalent existing patterns for master passwords and transient secrets.
- Do not expand filesystem or shell capabilities without a clear user-facing need and human review.
- Treat IPC changes that return plaintext secrets as security-sensitive.
