/** Wrappers de los comandos Tauri de estado persistido de la interfaz. */

import { invoke } from '@tauri-apps/api/core';

/**
 * Devuelve la última vista guardada como JSON sin parsear, o `null` si nunca
 * se guardó o si lo guardado dejó de ser válido.
 */
export function getViewState(): Promise<string | null> {
  return invoke<string | null>('get_view_state');
}

/** Guarda la vista actual. El backend solo valida que sea JSON. */
export function setViewState(json: string): Promise<void> {
  return invoke<void>('set_view_state', { json });
}
