/**
 * Aplica el tema elegido al documento.
 *
 * Siempre deja **una** de las dos clases en `<html>`, también en modo
 * `system`. Las dos hojas de estilo la necesitan: `tokens.css` pasa a oscuro
 * con `.dark` y a claro con `.light`, y la variante `dark:` de Tailwind solo
 * mira la clase. Dejar que cada una resolviera el modo sistema por su cuenta
 * las podía poner en desacuerdo.
 */

import type { ThemeMode } from '../types';

const DARK_QUERY = '(prefers-color-scheme: dark)';

/** Clave de la copia del tema que lee el script de `index.html` antes de pintar. */
export const THEME_STORAGE_KEY = 'mosaic.theme';

/**
 * Pone la clase del tema y, en modo `system`, la mantiene al día si el
 * sistema cambia de preferencia con la aplicación abierta.
 *
 * Devuelve la función que deja de escuchar al sistema; hay que llamarla
 * antes de aplicar otro modo.
 */
export function applyTheme(mode: ThemeMode): () => void {
  const root = document.documentElement;
  try {
    localStorage.setItem(THEME_STORAGE_KEY, mode);
  } catch {
    // Sin almacenamiento solo se pierde el arranque sin parpadeo.
  }

  const paint = (dark: boolean) => {
    root.classList.toggle('dark', dark);
    root.classList.toggle('light', !dark);
    // Los controles nativos (casillas, la lista de un `<select>`, las barras
    // de desplazamiento) no leen los tokens: siguen `color-scheme`.
    root.style.colorScheme = dark ? 'dark' : 'light';
  };

  if (mode !== 'system') {
    paint(mode === 'dark');
    return () => {};
  }

  const query = window.matchMedia(DARK_QUERY);
  const onChange = (event: MediaQueryListEvent) => paint(event.matches);
  paint(query.matches);
  query.addEventListener('change', onChange);
  return () => query.removeEventListener('change', onChange);
}
