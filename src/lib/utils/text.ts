/** Normalización de texto para búsquedas y comparaciones. */

/**
 * Deja un texto en la forma con la que se indexa y se busca: sin acentos y en
 * minúsculas.
 *
 * Se aplica a los dos lados de la comparación, de modo que «Ñandú» encuentra
 * «nandu» y al revés. La descomposición NFD separa cada letra de su tilde, y el
 * reemplazo se lleva las tildes sueltas.
 */
export function normalizeText(value: string): string {
  return value
    .normalize('NFD')
    .replace(/\p{Diacritic}/gu, '')
    .toLowerCase();
}

/**
 * Compara dos nombres como los ordena SQLite con `COLLATE NOCASE`, pero
 * respetando el alfabeto español (la ñ va entre la n y la o).
 */
export function compareNames(a: string, b: string): number {
  return a.localeCompare(b, 'es', { sensitivity: 'base' });
}
