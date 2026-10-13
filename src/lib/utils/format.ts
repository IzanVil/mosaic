/** Utilidades de presentación. */

import { get } from 'svelte/store';

import { lang, translate, type Lang } from '../i18n';

const MINUTE = 60;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;
const MONTH = 30 * DAY;
const YEAR = 365 * DAY;

/** Marca de tiempo actual en segundos desde el epoch Unix, como la de Rust. */
export function now_ts(): number {
  return Math.floor(Date.now() / 1000);
}

/**
 * Convierte una marca de tiempo Unix (segundos) en texto relativo al ahora, en
 * el idioma que se pida o en el actual. En una plantilla conviene pasar `$lang`
 * para que el texto cambie al cambiar de idioma.
 */
export function formatRelativeTime(timestamp: number | null, language: Lang = get(lang)): string {
  if (timestamp === null) return translate(language, 'tiempo.nunca');

  const seconds = Math.floor(Date.now() / 1000) - timestamp;
  if (seconds < MINUTE) return translate(language, 'tiempo.momento');
  if (seconds < HOUR) return translate(language, 'tiempo.minutos', { n: Math.floor(seconds / MINUTE) });
  if (seconds < DAY) return translate(language, 'tiempo.horas', { n: Math.floor(seconds / HOUR) });
  if (seconds < MONTH) return translate(language, 'tiempo.dias', { n: Math.floor(seconds / DAY) });
  if (seconds < YEAR) return translate(language, 'tiempo.meses', { n: Math.floor(seconds / MONTH) });
  return translate(language, 'tiempo.anios', { n: Math.floor(seconds / YEAR) });
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
