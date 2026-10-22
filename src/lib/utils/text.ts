/** Normalización de texto para búsquedas y comparaciones. */

/** Sin acentos y en minúsculas, en los dos lados de la búsqueda: «Ñandú» encuentra «nandu». */
export function normalizeText(value: string): string {
  return value
    .normalize('NFD')
    .replace(/\p{Diacritic}/gu, '')
    .toLowerCase();
}

/** Compara nombres como `COLLATE NOCASE` de SQLite, con la ñ entre la n y la o. */
export function compareNames(a: string, b: string): number {
  return a.localeCompare(b, 'es', { sensitivity: 'base' });
}
