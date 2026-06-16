<script lang="ts">
  import { onMount } from 'svelte';
  import type { Settings } from '../../ipc';
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

  const themeOptions = [
    { value: 'paper', label: 'paper' },
    { value: 'ink', label: 'ink' },
  ] as const;
  const autoLockOptions = [
    { value: '1', label: '1 min' },
    { value: '5', label: '5 min' },
    { value: '15', label: '15 min' },
    { value: '60', label: '60 min' },
    { value: 'never', label: 'never' },
  ];
  const clipboardOptions = [
    { value: '15', label: '15s' },
    { value: '30', label: '30s' },
    { value: '60', label: '60s' },
    { value: '120', label: '120s' },
  ];

  let autoLockEnabled = $state(true);
  let autoLockMinutes = $state(15);
  let clipboardEnabled = $state(true);
  let clipboardSeconds = $state(30);
  let fontSize = $state(13);
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
    if (!loaded || busy) {
      return;
    }

    clipboardEnabled = true;
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
        (!Number.isInteger(nextAutoLock) || nextAutoLock < 1 || nextAutoLock > 240)
      ) {
        errorMessage = 'Auto-lock timeout must be an integer between 1 and 240 minutes';
        return false;
      }

      const nextClipboard = clipboardEnabled ? Number(clipboardSeconds) : null;
      if (
        nextClipboard !== null &&
        (!Number.isInteger(nextClipboard) || nextClipboard < 5 || nextClipboard > 300 || nextClipboard % 5 !== 0)
      ) {
        errorMessage = 'Clipboard clear timeout must be a multiple of 5 between 5 and 300 seconds';
        return false;
      }

      const nextFontSize = Number(fontSize);
      if (!Number.isInteger(nextFontSize) || nextFontSize < 11 || nextFontSize > 16) {
        errorMessage = 'Font size must be an integer between 11 and 16';
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
    autoLockMinutes = Number(settings.autoLockTimeoutMinutes ?? 15);
    clipboardEnabled = settings.clipboardClearSeconds !== null && settings.clipboardClearSeconds !== undefined;
    clipboardSeconds = Number(settings.clipboardClearSeconds ?? 30);
    fontSize = settings.fontSize;
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
            options={[...themeOptions]}
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
            min={11}
            max={16}
            step={1}
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
          <Segmented ariaLabel="Auto-lock timeout" value={autoLockValue} options={autoLockOptions} onselect={setAutoLock} />
        </div>
      </div>

      <div class="set-row">
        <div class="set-row__k">clear clipboard<small>wipe copied secrets on the configured timer</small></div>
        <div class="set-row__v">
          <Toggle label="Clear clipboard automatically" checked={clipboardEnabled} ontoggle={setClipboardEnabled} />
          <Segmented
            ariaLabel="Clipboard clear timeout"
            value={clipboardValue}
            options={clipboardOptions}
            onselect={setClipboardDuration}
            class={clipboardEnabled ? '' : 'is-disabled'}
          />
        </div>
      </div>
    </div>

    <div class="set-group">
      <div class="set-group__title">vault</div>

      <div class="set-row">
        <div class="set-row__k">location<small>{vaultState.vaultPath || '~/.arca/vault.local'}</small></div>
        <div class="set-row__v">local</div>
      </div>

      <div class="set-row">
        <div class="set-row__k">kdf<small>argon2id · aes-512</small></div>
        <div class="set-row__v">tuned</div>
      </div>

      <div class="set-row">
        <div class="set-row__k">networking<small>local-first desktop build · no remote transport configured</small></div>
        <div class="set-row__v">offline by design</div>
      </div>
    </div>

    <div class="settings__actions">
      <Button variant="ghost" onclick={lockNow} disabled={!loaded || busy}>lock now</Button>
    </div>

    {#if errorMessage}
      <div class="settings-error mono error" role="alert">{errorMessage}</div>
    {/if}
  </div>
</section>
