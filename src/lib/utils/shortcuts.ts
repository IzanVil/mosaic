/** Atajos de teclado: reconocerlos es una función pura, para probarla sin ventana. */

import type { Key } from '../i18n';

/** Lo que puede pedir un atajo. */
export type ShortcutAction = 'palette' | 'refresh-git' | 'settings' | 'focus-search' | 'help';

/** Lo mínimo de un `KeyboardEvent` que hace falta para reconocer un atajo. */
export interface KeyPress {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  /** `true` si el foco está en un campo donde se escribe. */
  typing: boolean;
}

/** Un atajo, tal y como se enseña en la ayuda. */
export interface ShortcutInfo {
  /** Teclas, ya con la tecla de mando de cada sistema. */
  keys: string;
  /** Clave del texto que lo explica. */
  description: Extract<Key, `atajos.${string}`>;
}

/** En macOS el modificador es Cmd; en el resto, Ctrl. */
export function isMac(platform: string = navigator.userAgent): boolean {
  return /mac|iphone|ipad/i.test(platform);
}

/** ¿El foco está donde se escribe? Ahí «/» y «?» son letras, no atajos. */
export function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  if (target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement) return true;
  if (target instanceof HTMLInputElement) {
    return !['checkbox', 'radio', 'button', 'submit', 'range', 'color'].includes(target.type);
  }
  return false;
}

/** Qué atajo es esta tecla, o `null` si no es ninguno. */
export function matchShortcut(press: KeyPress, mac: boolean): ShortcutAction | null {
  // Con modificador valen también escribiendo; «/» y «?», no. Un modificador de más
  // (Ctrl+Shift+R) no cuenta: esas combinaciones son del sistema.
  const command = mac ? press.metaKey && !press.ctrlKey : press.ctrlKey && !press.metaKey;
  const key = press.key.toLowerCase();

  if (command && !press.altKey && !press.shiftKey) {
    if (key === 'k') return 'palette';
    if (key === 'r') return 'refresh-git';
    if (key === ',') return 'settings';
    return null;
  }

  if (press.ctrlKey || press.metaKey || press.altKey || press.typing) return null;
  if (press.key === '/') return 'focus-search';
  if (press.key === '?') return 'help';
  return null;
}

/** La lista que enseña la ayuda, con las teclas de este sistema. */
export function shortcutList(mac: boolean): ShortcutInfo[] {
  const mod = mac ? '⌘' : 'Ctrl+';
  return [
    { keys: `${mod}K`, description: 'atajos.paleta' },
    { keys: `${mod}R`, description: 'atajos.git' },
    { keys: `${mod},`, description: 'atajos.ajustes' },
    { keys: '/', description: 'atajos.buscar' },
    { keys: '?', description: 'atajos.ayuda' },
    { keys: 'Esc', description: 'atajos.esc' },
  ];
}
