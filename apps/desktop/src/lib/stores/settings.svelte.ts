import { getSettings, updateSettings, type Settings } from '../ipc';
import {
  DEFAULT_SETTINGS,
  normalizeSettings,
  themeForUi,
  uiThemeFor,
} from '../settings';
import { setThemePreference } from './ui.svelte';

export const runtimeSettings = $state({
  current: { ...DEFAULT_SETTINGS },
  loaded: false,
});

export async function loadRuntimeSettings(): Promise<Settings> {
  const settings = normalizeSettings(await getSettings());
  applyRuntimeSettings(settings);

  return settings;
}

export async function saveRuntimeSettings(settings: Settings): Promise<Settings> {
  const normalized = normalizeSettings(settings);
  await updateSettings(normalized);
  applyRuntimeSettings(normalized);

  return normalized;
}

export function applyRuntimeSettings(settings: Settings) {
  runtimeSettings.current = settings;
  runtimeSettings.loaded = true;
  setThemePreference(uiThemeFor(runtimeSettings.current.theme));
}

export { DEFAULT_SETTINGS, normalizeSettings, themeForUi, uiThemeFor };
