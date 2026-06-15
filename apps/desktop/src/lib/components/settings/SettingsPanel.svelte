<script lang="ts">
  import { onMount } from 'svelte';
  import { getSettings, updateSettings, type Settings, type Theme } from '../../ipc';
  import { setThemePreference, uiState, type ThemeName } from '../../stores/ui.svelte';
  import { Button, Kbd, Tag } from '../primitives';

  type ThemeOption = {
    key: ThemeName;
    label: string;
    surface: string;
    accent: string;
    meta: string;
  };

  const themeOptions: ThemeOption[] = [
    {
      key: 'paper',
      label: 'paper',
      surface: '#EBE5D6',
      accent: '#C8553D',
      meta: 'default',
    },
    {
      key: 'ink',
      label: 'ink',
      surface: '#14110D',
      accent: '#D86A52',
      meta: 'alternate',
    },
  ];

  let autoLockEnabled = $state(true);
  let autoLockMinutes = $state(15);
  let clipboardEnabled = $state(true);
  let clipboardSeconds = $state(30);
  let fontSize = $state(13);
  let busy = $state(false);
  let loaded = $state(false);
  let errorMessage = $state('');

  const settingsStatus = $derived(loaded ? (busy ? 'saving' : 'saved') : 'loading');
  const autoLockTag = $derived(autoLockEnabled ? `${autoLockMinutes}m` : 'off');

  onMount(() => {
    void loadSettings();
  });

  async function loadSettings() {
    busy = true;
    errorMessage = '';

    try {
      applySettings(await getSettings());
      loaded = true;
    } catch (error) {
      errorMessage = messageFromError(error);
    } finally {
      busy = false;
    }
  }

  async function selectTheme(theme: ThemeName) {
    if (theme === uiState.theme) {
      return;
    }

    if (await saveSettings(theme)) {
      setThemePreference(theme);
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

      await updateSettings({
        autoLockTimeoutMinutes: nextAutoLock,
        clipboardClearSeconds: nextClipboard,
        theme: themeForUi(theme),
        fontSize,
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
    setThemePreference(uiThemeFor(settings.theme));
  }

  function themeForUi(theme: ThemeName): Theme {
    return theme === 'ink' ? 'amber' : 'terminal';
  }

  function uiThemeFor(theme: Theme): ThemeName {
    return theme === 'amber' ? 'ink' : 'paper';
  }

  function messageFromError(error: unknown): string {
    if (typeof error === 'object' && error !== null && 'message' in error) {
      return String(error.message);
    }

    return 'Unable to update settings';
  }
</script>

<section class="settings-panel" aria-labelledby="settings-title">
  <div class="settings-head">
    <div>
      <div class="detail__crumbs mono">vault &nbsp;/&nbsp; <b>settings</b></div>
      <h1 id="settings-title" class="detail__title">settings<em>.</em></h1>
      <div class="detail__meta mono">
        <Tag value={settingsStatus} />
        <span>theme · <b>{uiState.theme}</b></span>
        <span>surface · <b>{uiState.theme === 'paper' ? 'warm' : 'dark'}</b></span>
      </div>
    </div>
    <Button variant="ghost" size="sm" onclick={() => (uiState.view = 'list')}>back_to_vault</Button>
  </div>

  <div class="settings-body">
    <section class="settings-group" aria-labelledby="settings-theme-title">
      <div class="settings-group__head">
        <div>
          <p class="settings-group__eyebrow mono">appearance</p>
          <h2 id="settings-theme-title">theme</h2>
        </div>
        <Tag variant={uiState.theme === 'ink' ? 'ink' : 'paper'} value={uiState.theme} />
      </div>

      <div class="theme-switch" role="group" aria-label="Theme">
        {#each themeOptions as option}
          <button
            type="button"
            class={uiState.theme === option.key ? 'theme-option theme-option--active' : 'theme-option'}
            aria-pressed={uiState.theme === option.key}
            onclick={() => selectTheme(option.key)}
            disabled={busy || !loaded}
          >
            <span class="theme-option__swatches" aria-hidden="true">
              <span class="theme-option__swatch" style:background={option.surface}></span>
              <span class="theme-option__swatch" style:background={option.accent}></span>
            </span>
            <span class="theme-option__label">{option.label}</span>
            <span class="theme-option__meta mono">{option.meta}</span>
          </button>
        {/each}
      </div>
    </section>

    <section class="settings-group settings-group--compact" aria-labelledby="settings-session-title">
      <div class="settings-group__head">
        <div>
          <p class="settings-group__eyebrow mono">session</p>
          <h2 id="settings-session-title">lock</h2>
        </div>
        <Tag variant="vault" value={autoLockTag} />
      </div>

      <label class="settings-row settings-row--control">
        <span>auto_lock</span>
        <span class="settings-control">
          <input bind:checked={autoLockEnabled} type="checkbox" onchange={() => void saveSettings()} disabled={busy || !loaded} />
          <input
            bind:value={autoLockMinutes}
            type="number"
            min="1"
            max="240"
            step="1"
            onchange={() => void saveSettings()}
            disabled={busy || !loaded || !autoLockEnabled}
          />
          <b>minutes</b>
        </span>
      </label>
      <label class="settings-row settings-row--control">
        <span>clipboard_clear</span>
        <span class="settings-control">
          <input bind:checked={clipboardEnabled} type="checkbox" onchange={() => void saveSettings()} disabled={busy || !loaded} />
          <input
            bind:value={clipboardSeconds}
            type="number"
            min="5"
            max="300"
            step="5"
            onchange={() => void saveSettings()}
            disabled={busy || !loaded || !clipboardEnabled}
          />
          <b>seconds</b>
        </span>
      </label>
      <div class="settings-row">
        <span>shortcut</span>
        <b><Kbd value="⌘" /> + <Kbd value="," /></b>
      </div>
      {#if errorMessage}
        <div class="settings-error mono error" role="alert">{errorMessage}</div>
      {/if}
    </section>
  </div>
</section>
