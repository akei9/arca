<script lang="ts">
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

  function selectTheme(theme: ThemeName) {
    setThemePreference(theme);
  }
</script>

<section class="settings-panel" aria-labelledby="settings-title">
  <div class="settings-head">
    <div>
      <div class="detail__crumbs mono">vault &nbsp;/&nbsp; <b>settings</b></div>
      <h1 id="settings-title" class="detail__title">settings<em>.</em></h1>
      <div class="detail__meta mono">
        <Tag value="appearance" />
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
        <Tag variant="vault" value="15m" />
      </div>

      <div class="settings-row">
        <span>auto_lock</span>
        <b>15 minutes</b>
      </div>
      <div class="settings-row">
        <span>clipboard_clear</span>
        <b>30 seconds</b>
      </div>
      <div class="settings-row">
        <span>shortcut</span>
        <b><Kbd value="⌘" /> + <Kbd value="," /></b>
      </div>
    </section>
  </div>
</section>
