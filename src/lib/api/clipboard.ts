/**
 * Portapapeles, a través del plugin oficial de Tauri.
 *
 * Vive en `api/` aunque no use `invoke()` directamente: el plugin habla con el
 * backend igual que un comando, y así ningún componente toca el puente.
 * La ventana solo tiene permiso para escribir texto, no para leerlo.
 */

import { writeText } from '@tauri-apps/plugin-clipboard-manager';

/** Copia un texto al portapapeles del sistema. */
export function copyText(text: string): Promise<void> {
  return writeText(text);
}
