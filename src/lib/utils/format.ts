/** Utilidades de presentación. */

const MINUTE = 60;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;
const MONTH = 30 * DAY;
const YEAR = 365 * DAY;

/**
 * Convierte una marca de tiempo Unix (segundos) en texto relativo al ahora.
 * Devuelve `'nunca'` si no hay marca.
 */
export function formatRelativeTime(timestamp: number | null): string {
  if (timestamp === null) return 'nunca';

  const seconds = Math.floor(Date.now() / 1000) - timestamp;
  if (seconds < MINUTE) return 'hace un momento';
  if (seconds < HOUR) return `hace ${plural(Math.floor(seconds / MINUTE), 'minuto')}`;
  if (seconds < DAY) return `hace ${plural(Math.floor(seconds / HOUR), 'hora')}`;
  if (seconds < MONTH) return `hace ${plural(Math.floor(seconds / DAY), 'día')}`;
  if (seconds < YEAR) return `hace ${plural(Math.floor(seconds / MONTH), 'mes', 'meses')}`;
  return `hace ${plural(Math.floor(seconds / YEAR), 'año')}`;
}

function plural(count: number, singular: string, plural = `${singular}s`): string {
  return `${count} ${count === 1 ? singular : plural}`;
}

/**
 * Acorta una ruta por la izquierda para que quepa en `maxLength` caracteres,
 * conservando siempre el final, que es la parte informativa.
 */
export function shortenPath(path: string, maxLength = 52): string {
  if (path.length <= maxLength) return path;
  return `…${path.slice(path.length - maxLength + 1)}`;
}

/** Formatea una duración en milisegundos de forma legible. */
export function formatDuration(milliseconds: number): string {
  if (milliseconds < 1000) return `${milliseconds} ms`;
  return `${(milliseconds / 1000).toFixed(1)} s`;
}
