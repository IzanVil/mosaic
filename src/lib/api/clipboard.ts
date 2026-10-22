/**
 * Portapapeles, por el plugin de Tauri. Va en `api/` aunque no use `invoke()`, para que
 * ningún componente toque el puente. Solo hay permiso para escribir, no para leer.
 */

import { writeText } from '@tauri-apps/plugin-clipboard-manager';

/** Copia un texto al portapapeles del sistema. */
export function copyText(text: string): Promise<void> {
  return writeText(text);
}
