/**
 * Atajos de teclado de Mosaic.
 *
 * Reconocer un atajo es una función pura de la tecla y del sitio donde está el
 * foco, para poder probarla sin una ventana. Quien la usa (`App.svelte`)
 * decide qué hacer con cada acción.
 */

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
  description: string;
}

/** En macOS el modificador es Cmd; en el resto, Ctrl. */
export function isMac(platform: string = navigator.userAgent): boolean {
  return /mac|iphone|ipad/i.test(platform);
}

/**
 * ¿El foco está en algo donde se escribe? Ahí «/» y «?» son letras, no
 * atajos.
 */
export function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  if (target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement) return true;
  if (target instanceof HTMLInputElement) {
    return !['checkbox', 'radio', 'button', 'submit', 'range', 'color'].includes(target.type);
  }
  return false;
}

/**
 * Qué atajo es esta tecla, o `null` si no es ninguno.
 *
 * Los de modificador (Ctrl+K, Ctrl+R, Ctrl+,) funcionan también escribiendo,
 * como en cualquier editor; los de una sola tecla («/» y «?»), no. Un
 * modificador de más (Ctrl+Shift+R, Ctrl+Alt+K) no cuenta como el atajo: deja
 * libres esas combinaciones para el sistema.
 */
export function matchShortcut(press: KeyPress, mac: boolean): ShortcutAction | null {
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
    { keys: `${mod}K`, description: 'Abrir la paleta de comandos' },
    { keys: `${mod}R`, description: 'Releer el estado de Git' },
    { keys: `${mod},`, description: 'Ir a Ajustes' },
    { keys: '/', description: 'Buscar en el tablero' },
    { keys: '?', description: 'Ver esta lista de atajos' },
    { keys: 'Esc', description: 'Cerrar lo que esté abierto, o volver del detalle' },
  ];
}
