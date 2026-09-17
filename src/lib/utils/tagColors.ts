export interface TagColor {
  name: string;
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
export const DEFAULT_TAG_COLOR = '#3B82F6';
const CHIP_BACKGROUND_ALPHA = '26';
const CHIP_BORDER_ALPHA = '4D';

function withAlpha(hex: string, alpha: string): string {
  return /^#[0-9A-Fa-f]{6}$/.test(hex) ? `${hex}${alpha}` : hex;
}
export function chipBackground(hex: string): string {
  return withAlpha(hex, CHIP_BACKGROUND_ALPHA);
}
export function chipBorder(hex: string): string {
  return withAlpha(hex, CHIP_BORDER_ALPHA);
}
export function colorInputValue(hex: string): string {
  return /^#[0-9A-Fa-f]{6}$/.test(hex) ? hex.toLowerCase() : DEFAULT_TAG_COLOR.toLowerCase();
}
