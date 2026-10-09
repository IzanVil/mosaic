/**
 * El tema se guarda también en localStorage para que el script de
 * `index.html` lo aplique antes de pintar. Si las dos mitades no usan la misma
 * clave y los mismos valores, vuelve el parpadeo sin que nada falle.
 */

import { readFileSync } from 'node:fs';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { THEME_STORAGE_KEY, applyTheme } from '../../src/lib/utils/theme';

function fakeDom(stored: Record<string, string>, systemDark = false, storageThrows = false) {
  const classes = new Set<string>();
  const root = {
    classList: {
      toggle: (name: string, on: boolean) => (on ? classes.add(name) : classes.delete(name)),
    },
    style: {} as Record<string, string>,
  };
  vi.stubGlobal('document', { documentElement: root });
  const media = () => ({
    matches: systemDark,
    addEventListener: () => {},
    removeEventListener: () => {},
  });
  vi.stubGlobal('window', { matchMedia: media });
  vi.stubGlobal('matchMedia', media);
  vi.stubGlobal('localStorage', {
    getItem: (key: string) => {
      if (storageThrows) throw new Error('bloqueado');
      return stored[key] ?? null;
    },
    setItem: (key: string, value: string) => {
      if (storageThrows) throw new Error('bloqueado');
      stored[key] = value;
    },
  });
  return { classes, root };
}

function runBootScript(): void {
  const html = readFileSync('index.html', 'utf8');
  const script = html.match(/<script>([\s\S]*?)<\/script>/)?.[1];
  expect(script).toBeDefined();
  new Function(script!)();
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('copia del tema para el arranque', () => {
  it('aplicar un tema lo guarda en la clave que lee index.html', () => {
    const stored: Record<string, string> = {};
    fakeDom(stored);
    applyTheme('light');
    expect(stored[THEME_STORAGE_KEY]).toBe('light');
  });

  it('el script de arranque pinta el tema guardado', () => {
    for (const [mode, systemDark, expected] of [
      ['light', true, 'light'],
      ['dark', false, 'dark'],
      ['system', true, 'dark'],
      ['system', false, 'light'],
    ] as const) {
      const { classes, root } = fakeDom({ [THEME_STORAGE_KEY]: mode }, systemDark);
      runBootScript();
      expect([...classes], mode).toEqual([expected]);
      expect(root.style.colorScheme).toBe(expected);
    }
  });

  it('sin nada guardado arranca en oscuro, como el tema por defecto', () => {
    const { classes } = fakeDom({});
    runBootScript();
    expect([...classes]).toEqual(['dark']);
  });

  it('sin almacenamiento disponible no rompe ni el arranque ni el cambio de tema', () => {
    const { classes } = fakeDom({}, false, true);
    expect(() => runBootScript()).not.toThrow();
    expect(() => applyTheme('light')).not.toThrow();
    expect([...classes]).toEqual(['light']);
  });
});
