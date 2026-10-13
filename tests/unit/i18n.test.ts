/** El idioma de la interfaz y la función de traducir. */

import { get } from 'svelte/store';
import { afterEach, describe, expect, it } from 'vitest';

import { en } from '../../src/lib/i18n/en';
import { es } from '../../src/lib/i18n/es';
import { lang, langPreference, systemLang, t, translate } from '../../src/lib/i18n';
import { sanitizeViewState } from '../../src/lib/stores/filters';
import { formatRelativeTime } from '../../src/lib/utils/format';

afterEach(() => langPreference.set('system'));

describe('diccionarios', () => {
  it('el inglés tiene exactamente las claves del español', () => {
    expect(Object.keys(en).sort()).toEqual(Object.keys(es).sort());
  });

  it('ningún texto está vacío', () => {
    for (const dict of [es, en]) {
      for (const [clave, valor] of Object.entries(dict)) {
        const texto = typeof valor === 'function' ? (valor as (p: unknown) => string)({ n: 2, nombre: 'x', cuando: 'x', visibles: 1, total: 2, actual: 'a', siguiente: 'b', lenguaje: 'x', estado: 'x', duracion: '1 s' }) : valor;
        expect(texto.trim(), clave).not.toBe('');
      }
    }
  });
});

describe('traducir', () => {
  it('resuelve los plurales de cada idioma', () => {
    expect(translate('es', 'app.numProyectos', { n: 1 })).toBe('1 proyecto');
    expect(translate('es', 'app.numProyectos', { n: 3 })).toBe('3 proyectos');
    expect(translate('en', 'app.numProyectos', { n: 1 })).toBe('1 project');
    expect(translate('en', 'app.numProyectos', { n: 3 })).toBe('3 projects');
  });

  it('el store sigue a la preferencia', () => {
    langPreference.set('en');
    expect(get(t)('app.ajustes')).toBe('Settings');
    langPreference.set('es');
    expect(get(t)('app.ajustes')).toBe('Ajustes');
  });

  it('las fechas relativas salen en el idioma pedido', () => {
    const hace = Math.floor(Date.now() / 1000) - 3 * 3600;
    expect(formatRelativeTime(hace, 'es')).toBe('hace 3 horas');
    expect(formatRelativeTime(hace, 'en')).toBe('3 hours ago');
    expect(formatRelativeTime(null, 'en')).toBe('never');
  });
});

describe('idioma del sistema', () => {
  it('coge el primero que Mosaic habla', () => {
    expect(systemLang(['es-ES', 'en-US'])).toBe('es');
    expect(systemLang(['fr-FR', 'en-GB', 'es'])).toBe('en');
    expect(systemLang(['es_ES.UTF-8'])).toBe('es');
  });

  it('si no habla ninguno, inglés', () => {
    expect(systemLang(['fr', 'de'])).toBe('en');
    expect(systemLang([])).toBe('en');
  });

  it('una preferencia explícita gana al sistema', () => {
    langPreference.set('es');
    expect(get(lang)).toBe('es');
  });
});

describe('preferencia guardada', () => {
  it('una vista de una versión anterior arranca en el idioma del sistema', () => {
    expect(sanitizeViewState({ theme: 'light' }).lang).toBe('system');
  });

  it('un valor desconocido no rompe el arranque', () => {
    expect(sanitizeViewState({ lang: 'fr' }).lang).toBe('system');
    expect(sanitizeViewState({ lang: 'en' }).lang).toBe('en');
  });
});
