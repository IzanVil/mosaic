/**
 * Qué tecla es qué atajo. Lo importante es lo que NO es atajo: «/» y «?» no
 * pueden robarle la tecla a quien está escribiendo, y una combinación con un
 * modificador de más no puede dispararlo por accidente.
 */

import { describe, expect, it } from 'vitest';

import { matchShortcut, shortcutList, type KeyPress } from '../../src/lib/utils/shortcuts';

const press = (key: string, extra: Partial<KeyPress> = {}): KeyPress => ({
  key,
  ctrlKey: false,
  metaKey: false,
  altKey: false,
  shiftKey: false,
  typing: false,
  ...extra,
});

describe('matchShortcut en Linux y Windows', () => {
  it('reconoce los tres atajos con Ctrl, también escribiendo', () => {
    expect(matchShortcut(press('k', { ctrlKey: true }), false)).toBe('palette');
    expect(matchShortcut(press('K', { ctrlKey: true }), false)).toBe('palette');
    expect(matchShortcut(press('r', { ctrlKey: true, typing: true }), false)).toBe('refresh-git');
    expect(matchShortcut(press(',', { ctrlKey: true }), false)).toBe('settings');
  });

  it('no usa Cmd ni acepta modificadores de más', () => {
    expect(matchShortcut(press('k', { metaKey: true }), false)).toBeNull();
    expect(matchShortcut(press('r', { ctrlKey: true, shiftKey: true }), false)).toBeNull();
    expect(matchShortcut(press('k', { ctrlKey: true, altKey: true }), false)).toBeNull();
  });

  it('reconoce «/» y «?» solo cuando no se está escribiendo', () => {
    expect(matchShortcut(press('/'), false)).toBe('focus-search');
    expect(matchShortcut(press('?', { shiftKey: true }), false)).toBe('help');
    expect(matchShortcut(press('/', { typing: true }), false)).toBeNull();
    expect(matchShortcut(press('?', { shiftKey: true, typing: true }), false)).toBeNull();
  });

  it('ignora las teclas sueltas que no son atajo', () => {
    for (const key of ['k', 'r', 'Escape', 'Enter', 'a']) {
      expect(matchShortcut(press(key), false), key).toBeNull();
    }
  });
});

describe('matchShortcut en macOS', () => {
  it('usa Cmd y no Ctrl', () => {
    expect(matchShortcut(press('k', { metaKey: true }), true)).toBe('palette');
    expect(matchShortcut(press('k', { ctrlKey: true }), true)).toBeNull();
  });
});

describe('la ayuda', () => {
  it('enseña las teclas del sistema en el que está', () => {
    expect(shortcutList(false)[0]?.keys).toBe('Ctrl+K');
    expect(shortcutList(true)[0]?.keys).toBe('⌘K');
  });
});
