/** Wrappers de los comandos Tauri de rutas de escaneo. */

import { invoke } from '@tauri-apps/api/core';

import type { ScanPath, ScanSummary } from '../types';

/** Registra una ruta raíz. La ruta se canonicaliza en el backend. */
export function addScanPath(path: string): Promise<ScanPath> {
  return invoke<ScanPath>('add_scan_path', { path });
}

/** Devuelve todas las rutas raíz configuradas. */
export function listScanPaths(): Promise<ScanPath[]> {
  return invoke<ScanPath[]>('list_scan_paths');
}

/** Elimina una ruta raíz. Los proyectos ya descubiertos se conservan. */
export function removeScanPath(id: number): Promise<boolean> {
  return invoke<boolean>('remove_scan_path', { id });
}

/** Habilita o deshabilita una ruta raíz sin borrarla. */
export function setScanPathEnabled(id: number, enabled: boolean): Promise<void> {
  return invoke<void>('set_scan_path_enabled', { id, enabled });
}

/** Escanea todas las rutas habilitadas y devuelve el resumen. */
export function scanAllPaths(): Promise<ScanSummary> {
  return invoke<ScanSummary>('scan_all_paths');
}
