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

export interface TagColor {
  /** Nombre visible, para el `title` y el texto accesible. */
  name: string;
  /** Hexadecimal en mayúsculas, la forma que normaliza el backend. */
  hex: string;
}

export const TAG_COLORS: readonly TagColor[] = [
  { name: 'Pizarra', hex: '#64748B' },
  { name: 'Gris', hex: '#6B7280' },
  { name: 'Rojo', hex: '#EF4444' },
  { name: 'Naranja', hex: '#F97316' },
  { name: 'Ámbar', hex: '#F59E0B' },
  { name: 'Amarillo', hex: '#EAB308' },
  { name: 'Lima', hex: '#84CC16' },
  { name: 'Verde', hex: '#22C55E' },
  { name: 'Esmeralda', hex: '#10B981' },
  { name: 'Turquesa', hex: '#14B8A6' },
  { name: 'Cian', hex: '#06B6D4' },
  { name: 'Celeste', hex: '#0EA5E9' },
  { name: 'Azul', hex: '#3B82F6' },
  { name: 'Índigo', hex: '#6366F1' },
  { name: 'Violeta', hex: '#8B5CF6' },
  { name: 'Fucsia', hex: '#D946EF' },
] as const;

/** Color con el que se crea una etiqueta si el usuario no elige otro. */
export const DEFAULT_TAG_COLOR = '#3B82F6';

/** Valor válido para `<input type="color">`, que solo acepta `#rrggbb`. */
export function colorInputValue(hex: string): string {
  return /^#[0-9A-Fa-f]{6}$/.test(hex) ? hex.toLowerCase() : DEFAULT_TAG_COLOR.toLowerCase();
}
