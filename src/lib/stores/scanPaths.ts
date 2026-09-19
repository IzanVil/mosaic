/** Estado global de las rutas raíz configuradas. */

import { writable } from 'svelte/store';

import * as api from '../api/scanner';
import type { ScanPath } from '../types';

export const scanPaths = writable<ScanPath[]>([]);
export const loadingScanPaths = writable(false);
/** Último error de gestión de rutas, para mostrarlo en la interfaz. */
export const scanPathsError = writable<string | null>(null);

async function refresh(): Promise<void> {
  scanPaths.set(await api.listScanPaths());
}

/** Recarga las rutas configuradas. */
export async function loadScanPaths(): Promise<void> {
  loadingScanPaths.set(true);
  scanPathsError.set(null);
  try {
    await refresh();
  } catch (error) {
    scanPathsError.set(String(error));
  } finally {
    loadingScanPaths.set(false);
  }
}

/** Añade una ruta raíz. Devuelve `true` si se registró. */
export async function addScanPath(path: string): Promise<boolean> {
  scanPathsError.set(null);
  try {
    await api.addScanPath(path);
    await refresh();
    return true;
  } catch (error) {
    scanPathsError.set(String(error));
    return false;
  }
}

/** Elimina una ruta raíz. */
export async function removeScanPath(id: number): Promise<void> {
  scanPathsError.set(null);
  try {
    await api.removeScanPath(id);
    await refresh();
  } catch (error) {
    scanPathsError.set(String(error));
  }
}

/** Habilita o deshabilita una ruta raíz. */
export async function setScanPathEnabled(id: number, enabled: boolean): Promise<void> {
  scanPathsError.set(null);
  try {
    await api.setScanPathEnabled(id, enabled);
    await refresh();
  } catch (error) {
    scanPathsError.set(String(error));
  }
}
