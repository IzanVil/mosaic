/**
 * Paleta de colores de etiqueta.
 *
 * Son los tonos 500 de la paleta por defecto de Tailwind: se llevan bien con
 * los tokens de `app.css` en claro y en oscuro, y al estar escritos aquí como
 * literales no dependen de que Tailwind genere ninguna clase.
 *
 * El backend guarda hexadecimal, así que el selector libre (`<input
 * type="color">`) del gestor de etiquetas admite cualquier otro valor.
 */

/** Identificador de un color; su nombre visible sale del diccionario (`color.<id>`). */
export type TagColorId =
  | 'slate' | 'gray' | 'red' | 'orange' | 'amber' | 'yellow' | 'lime' | 'green'
  | 'emerald' | 'teal' | 'cyan' | 'sky' | 'blue' | 'indigo' | 'violet' | 'fuchsia';

export interface TagColor {
  id: TagColorId;
  /** Hexadecimal en mayúsculas, la forma que normaliza el backend. */
  hex: string;
}

export const TAG_COLORS: readonly TagColor[] = [
  { id: 'slate', hex: '#64748B' },
  { id: 'gray', hex: '#6B7280' },
  { id: 'red', hex: '#EF4444' },
  { id: 'orange', hex: '#F97316' },
  { id: 'amber', hex: '#F59E0B' },
  { id: 'yellow', hex: '#EAB308' },
  { id: 'lime', hex: '#84CC16' },
  { id: 'green', hex: '#22C55E' },
  { id: 'emerald', hex: '#10B981' },
  { id: 'teal', hex: '#14B8A6' },
  { id: 'cyan', hex: '#06B6D4' },
  { id: 'sky', hex: '#0EA5E9' },
  { id: 'blue', hex: '#3B82F6' },
  { id: 'indigo', hex: '#6366F1' },
  { id: 'violet', hex: '#8B5CF6' },
  { id: 'fuchsia', hex: '#D946EF' },
] as const;

/** Color con el que se crea una etiqueta si el usuario no elige otro. */
export const DEFAULT_TAG_COLOR = '#3B82F6';

/** Valor válido para `<input type="color">`, que solo acepta `#rrggbb`. */
export function colorInputValue(hex: string): string {
  return /^#[0-9A-Fa-f]{6}$/.test(hex) ? hex.toLowerCase() : DEFAULT_TAG_COLOR.toLowerCase();
}
