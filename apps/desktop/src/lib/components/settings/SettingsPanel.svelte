<script lang="ts">
  import { onMount } from 'svelte';
  import type { Settings } from '../../ipc';
  import {
    RELEASE_AUTO_LOCK_OPTIONS,
    RELEASE_CLIPBOARD_CLEAR_OPTIONS,
    RELEASE_THEME_OPTIONS,
    SETTINGS_LIMITS,
  } from '../../settings';
  import { lockCurrentVault } from '../../session';
  import {
    loadRuntimeSettings,
    runtimeSettings,
    saveRuntimeSettings,
    themeForUi,
  } from '../../stores/settings.svelte';
  import { uiState, type ThemeName } from '../../stores/ui.svelte';
  import { vaultState } from '../../stores/vault.svelte';
  import { Button, Segmented, Slider, Toggle } from '../primitives';

  let autoLockEnabled = $state(true);
  let autoLockMinutes = $state<number>(SETTINGS_LIMITS.autoLockTimeoutMinutes.defaultValue);
  let clipboardEnabled = $state(true);
  let clipboardSeconds = $state<number>(SETTINGS_LIMITS.clipboardClearSeconds.defaultValue);
  let fontSize = $state<number>(SETTINGS_LIMITS.fontSize.defaultValue);
  let busy = $state(false);
  let loaded = $state(false);
  let errorMessage = $state('');

  const autoLockValue = $derived(autoLockEnabled ? String(autoLockMinutes) : 'never');
  const clipboardValue = $derived(String(clipboardSeconds));

  onMount(() => {
    if (runtimeSettings.loaded) {
      applySettings(runtimeSettings.current);
      loaded = true;
      return;
    }

    void loadSettings();
  });

  async function loadSettings() {
    busy = true;
    errorMessage = '';

    try {
      applySettings(await loadRuntimeSettings());
      loaded = true;
    } catch (error) {
      errorMessage = messageFromError(error);
    } finally {
      busy = false;
    }
  }

  async function setTheme(theme: string) {
    if (!loaded || busy) {
      return;
    }

    await saveSettings(theme as ThemeName);
  }

  async function setAutoLock(value: string) {
    if (!loaded || busy) {
      return;
    }

    if (value === 'never') {
      autoLockEnabled = false;
    } else {
      autoLockEnabled = true;
      autoLockMinutes = Number(value);
    }

    await saveSettings();
  }

  async function setClipboardEnabled(next: boolean) {
    if (!loaded || busy) {
      return;
    }

    clipboardEnabled = next;
    await saveSettings();
  }

  async function setClipboardDuration(value: string) {
    if (!loaded || busy || !clipboardEnabled) {
      return;
    }

    clipboardSeconds = Number(value);
    await saveSettings();
  }

  async function setFontSize(next: number) {
    if (!loaded || busy) {
      return;
    }

    fontSize = next;
    await saveSettings();
  }

  async function lockNow() {
    if (busy) {
      return;
    }

    busy = true;
    errorMessage = '';

    try {
      await lockCurrentVault();
    } catch (error) {
      errorMessage = messageFromError(error);
    } finally {
      busy = false;
    }
  }

  async function saveSettings(theme = uiState.theme): Promise<boolean> {
    if (!loaded) {
      errorMessage = 'Settings must load before changes can be saved';
      return false;
    }

    busy = true;
    errorMessage = '';

    try {
      const nextAutoLock = autoLockEnabled ? Number(autoLockMinutes) : null;
      if (
        nextAutoLock !== null &&
        (!Number.isInteger(nextAutoLock) ||
          nextAutoLock < SETTINGS_LIMITS.autoLockTimeoutMinutes.min ||
          nextAutoLock > SETTINGS_LIMITS.autoLockTimeoutMinutes.max)
      ) {
        errorMessage = `Auto-lock timeout must be an integer between ${SETTINGS_LIMITS.autoLockTimeoutMinutes.min} and ${SETTINGS_LIMITS.autoLockTimeoutMinutes.max} minutes`;
        return false;
      }

      const nextClipboard = clipboardEnabled ? Number(clipboardSeconds) : null;
      if (
        nextClipboard !== null &&
        (!Number.isInteger(nextClipboard) ||
          nextClipboard < SETTINGS_LIMITS.clipboardClearSeconds.min ||
          nextClipboard > SETTINGS_LIMITS.clipboardClearSeconds.max ||
          nextClipboard % SETTINGS_LIMITS.clipboardClearSeconds.step !== 0)
      ) {
        errorMessage = `Clipboard clear timeout must be a multiple of ${SETTINGS_LIMITS.clipboardClearSeconds.step} between ${SETTINGS_LIMITS.clipboardClearSeconds.min} and ${SETTINGS_LIMITS.clipboardClearSeconds.max} seconds`;
        return false;
      }

      const nextFontSize = Number(fontSize);
      if (
        !Number.isInteger(nextFontSize) ||
        nextFontSize < SETTINGS_LIMITS.fontSize.min ||
        nextFontSize > SETTINGS_LIMITS.fontSize.max
      ) {
        errorMessage = `Font size must be an integer between ${SETTINGS_LIMITS.fontSize.min} and ${SETTINGS_LIMITS.fontSize.max}`;
        return false;
      }

      await saveRuntimeSettings({
        autoLockTimeoutMinutes: nextAutoLock,
        clipboardClearSeconds: nextClipboard,
        theme: themeForUi(theme),
        fontSize: nextFontSize,
      });
      return true;
    } catch (error) {
      errorMessage = messageFromError(error);
      return false;
    } finally {
      busy = false;
    }
  }

  function applySettings(settings: Settings) {
    autoLockEnabled = settings.autoLockTimeoutMinutes !== null && settings.autoLockTimeoutMinutes !== undefined;
    autoLockMinutes = normalizeInteger(
      settings.autoLockTimeoutMinutes ?? SETTINGS_LIMITS.autoLockTimeoutMinutes.defaultValue,
      SETTINGS_LIMITS.autoLockTimeoutMinutes.defaultValue,
      SETTINGS_LIMITS.autoLockTimeoutMinutes.min,
      SETTINGS_LIMITS.autoLockTimeoutMinutes.max,
      SETTINGS_LIMITS.autoLockTimeoutMinutes.step,
    );
    clipboardEnabled = settings.clipboardClearSeconds !== null && settings.clipboardClearSeconds !== undefined;
    clipboardSeconds = normalizeInteger(
      settings.clipboardClearSeconds ?? SETTINGS_LIMITS.clipboardClearSeconds.defaultValue,
      SETTINGS_LIMITS.clipboardClearSeconds.defaultValue,
      SETTINGS_LIMITS.clipboardClearSeconds.min,
      SETTINGS_LIMITS.clipboardClearSeconds.max,
      SETTINGS_LIMITS.clipboardClearSeconds.step,
    );
    fontSize = normalizeInteger(
      settings.fontSize,
      SETTINGS_LIMITS.fontSize.defaultValue,
      SETTINGS_LIMITS.fontSize.min,
      SETTINGS_LIMITS.fontSize.max,
      SETTINGS_LIMITS.fontSize.step,
    );
  }

  function normalizeInteger(value: unknown, fallback: number, min: number, max: number, multiple = 1): number {
    const next = Number(value);
    return Number.isInteger(next) && next >= min && next <= max && next % multiple === 0 ? next : fallback;
  }

  function messageFromError(error: unknown): string {
    if (typeof error === 'object' && error !== null && 'message' in error) {
      return String(error.message);
    }

    return 'Unable to update settings';
  }
</script>

<section class="page settings-page" aria-labelledby="settings-title">
  <div class="page__head">
    <span class="page__hash mono">#</span>
    <h1 id="settings-title" class="page__title">settings<em>.</em></h1>
    <div class="page__meta mono">
      <span>vault · <b>{vaultState.vaultName || 'vault.local'}</b></span>
      <span>theme · <b>{uiState.theme}</b></span>
    </div>
  </div>

  <div class="settings">
    <div class="set-group">
      <div class="set-group__title">appearance</div>

      <div class="set-row">
        <div class="set-row__k">theme<small>paper is the default surface · ink for low light</small></div>
        <div class="set-row__v">
          <Segmented
            ariaLabel="Theme"
            value={uiState.theme}
            options={[...RELEASE_THEME_OPTIONS]}
            onselect={setTheme}
          />
        </div>
      </div>

      <div class="set-row">
        <div class="set-row__k">font size<small>applies across lists, detail, and settings surfaces</small></div>
        <div class="set-row__v set-row__v--wide">
          <Slider
            ariaLabel="Font size"
            value={fontSize}
            min={SETTINGS_LIMITS.fontSize.min}
            max={SETTINGS_LIMITS.fontSize.max}
            step={SETTINGS_LIMITS.fontSize.step}
            valueLabel={`${fontSize}px`}
            oninput={(next) => {
              fontSize = next;
            }}
            onchange={setFontSize}
          />
        </div>
      </div>
    </div>

    <div class="set-group">
      <div class="set-group__title">security</div>

      <div class="set-row">
        <div class="set-row__k">auto-lock<small>seal the vault after inactivity</small></div>
        <div class="set-row__v">
          <Segmented
            ariaLabel="Auto-lock timeout"
            value={autoLockValue}
            options={[...RELEASE_AUTO_LOCK_OPTIONS]}
            onselect={setAutoLock}
          />
        </div>
      </div>

      <div class="set-row">
        <div class="set-row__k">clear clipboard<small>wipe copied secrets on the configured timer</small></div>
        <div class="set-row__v">
          <Toggle label="Clear clipboard automatically" checked={clipboardEnabled} ontoggle={setClipboardEnabled} />
          {#if clipboardEnabled}
            <Segmented
              ariaLabel="Clipboard clear timeout"
              value={clipboardValue}
              options={[...RELEASE_CLIPBOARD_CLEAR_OPTIONS]}
              onselect={setClipboardDuration}
            />
          {/if}
        </div>
      </div>
    </div>

    <div class="settings__actions">
      <Button variant="ghost" onclick={lockNow} disabled={!loaded || busy} aria-keyshortcuts="Meta+Shift+L Control+Shift+L">lock now</Button>
    </div>

    {#if errorMessage}
      <div class="settings-error mono error" role="alert">{errorMessage}</div>
    {/if}
  </div>
</section>
