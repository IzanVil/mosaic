/** Wrappers de los comandos Tauri de integración con el sistema. */

import { invoke } from '@tauri-apps/api/core';

import type { AppKind, DetectedApp, PreferredApps } from '../types';

/** Devuelve los IDEs y terminales reconocidos que hay instalados. */
export function listDetectedApps(): Promise<DetectedApp[]> {
  return invoke<DetectedApp[]>('list_detected_apps');
}

/** Abre un proyecto en el IDE, la terminal o el explorador de archivos. */
export function openIn(kind: AppKind, projectId: number): Promise<void> {
  return invoke<void>('open_in', { kind, projectId });
}

/** Devuelve las aplicaciones preferidas guardadas. */
export function getPreferredApps(): Promise<PreferredApps> {
  return invoke<PreferredApps>('get_preferred_apps');
}

/** Guarda el IDE o el terminal preferido. Cadena vacía = el primero disponible. */
export function setPreferredApp(kind: AppKind, appId: string): Promise<void> {
  return invoke<void>('set_preferred_app', { kind, appId });
}
