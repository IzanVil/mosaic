/** Diálogos nativos de abrir y guardar, por el plugin de Tauri; `null` si se cancela. */

import { open, save } from '@tauri-apps/plugin-dialog';

/** Filtro de un diálogo: un nombre y las extensiones que deja ver. */
export interface FileFilter {
  name: string;
  extensions: string[];
}

/** Pide una carpeta. */
export async function chooseFolder(): Promise<string | null> {
  const selected = await open({ directory: true, multiple: false });
  return typeof selected === 'string' ? selected : null;
}

/** Pide un fichero que abrir. */
export async function chooseFileToOpen(filters: FileFilter[]): Promise<string | null> {
  const selected = await open({ directory: false, multiple: false, filters });
  return typeof selected === 'string' ? selected : null;
}

/** Pide dónde guardar un fichero, proponiendo `defaultPath`. */
export function chooseSaveLocation(
  defaultPath: string,
  filters: FileFilter[],
): Promise<string | null> {
  return save({ defaultPath, filters });
}
