export function normalizeText(value: string): string {
  return value
    .normalize('NFD')
    .replace(/\p{Diacritic}/gu, '')
    .toLowerCase();
}
export function compareNames(a: string, b: string): number {
  return a.localeCompare(b, 'es', { sensitivity: 'base' });
}
