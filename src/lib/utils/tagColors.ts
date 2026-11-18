/**
 * Colores de etiqueta: los tonos 500 de Tailwind, escritos como literales para no
 * depender de sus clases. El selector libre del gestor admite cualquier otro.
 */

/** Identificador de un color; su nombre visible sale del diccionario (`color.<id>`). */
export type TagColorId =
  | 'slate' | 'gray' | 'red' | 'orange' | 'amber' | 'yellow' | 'lime' | 'green'
  | 'emerald' | 'teal' | 'cyan' | 'sky' | 'blue' | 'indigo' | 'violet' | 'fuchsia';

/** Un color de la paleta. */
export interface TagColor {
  id: TagColorId;
  /** Hexadecimal en mayúsculas, la forma que normaliza el backend. */
  hex: string;
}

/** Los dieciséis colores, en el orden de las muestras. */
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

/**
 * La cenefa de una tarjeta: un tramo por etiqueta, en su orden y del mismo ancho,
 * separados por una llaga transparente de `--tile-grout`. Sin etiquetas no hay
 * cenefa propia: la tarjeta pone la neutra.
 */
export function tagBand(colors: readonly string[]): string | undefined {
  if (colors.length === 0) return undefined;
  if (colors.length === 1) return colors[0];
  const at = (i: number) => Number(((i * 100) / colors.length).toFixed(3));
  const stops = colors.map((color, i) =>
    i === colors.length - 1
      ? `${color} ${at(i)}% 100%`
      : `${color} ${at(i)}% calc(${at(i + 1)}% - var(--tile-grout)), transparent 0 ${at(i + 1)}%`,
  );
  return `linear-gradient(90deg, ${stops.join(', ')})`;
}
