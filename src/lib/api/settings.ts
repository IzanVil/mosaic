/** Wrappers de los comandos Tauri de la vista guardada y los ajustes avanzados. */

import { invoke } from '@tauri-apps/api/core';

import type { AdvancedSettings, AdvancedSettingsView } from '../types';

/** La última vista guardada, en JSON sin parsear; `null` si no hay o ya no vale. */
export function getViewState(): Promise<string | null> {
  return invoke<string | null>('get_view_state');
}

/** Guarda la vista actual. El backend solo valida que sea JSON. */
export function setViewState(json: string): Promise<void> {
  return invoke<void>('set_view_state', { json });
}

/** Devuelve los ajustes avanzados con sus valores de fábrica y sus límites. */
export function getAdvancedSettings(): Promise<AdvancedSettingsView> {
  return invoke<AdvancedSettingsView>('get_advanced_settings');
}

/** Valida y guarda los ajustes avanzados, y devuelve lo que ha quedado guardado. */
export function setAdvancedSettings(settings: AdvancedSettings): Promise<AdvancedSettings> {
  return invoke<AdvancedSettings>('set_advanced_settings', { settings });
}
