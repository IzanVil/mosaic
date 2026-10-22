/**
 * Aplica el tema al documento, dejando siempre `.dark` o `.light` en `<html>`, también
 * en modo sistema: si `tokens.css` y Tailwind lo resolvieran cada uno, podrían discrepar.
 */

import type { ThemeMode } from '../types';

const DARK_QUERY = '(prefers-color-scheme: dark)';

/** Clave de la copia del tema que lee el script de `index.html` antes de pintar. */
export const THEME_STORAGE_KEY = 'mosaic.theme';

/** Pone la clase del tema y, en `system`, la sigue; devuelve cómo dejar de escuchar. */
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
    // Casillas, listas de `<select>` y barras de desplazamiento no leen los tokens: siguen esto.
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
