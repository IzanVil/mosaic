/**
 * Errores del backend en la interfaz: en español, el mensaje del backend; en
 * otro idioma, el texto de su código. Y que el backend no tenga códigos sin
 * texto en inglés, ni datos que el texto espere y no lleguen.
 */

import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';

import { describe, expect, it } from 'vitest';

import { errorTexts } from '../../src/lib/i18n/errors';
import type { CommandError } from '../../src/lib/types';
import { errorText, isCommandError } from '../../src/lib/utils/errors';

const taken: CommandError = {
  code: 'tag_name_taken',
  params: { name: 'Cliente' },
  message: 'ya existe una etiqueta llamada «Cliente»',
};

describe('texto de un error', () => {
  it('en español enseña el mensaje del backend tal cual', () => {
    expect(errorText(taken, 'es')).toBe('ya existe una etiqueta llamada «Cliente»');
  });

  it('en inglés lo traduce con sus datos', () => {
    expect(errorText(taken, 'en')).toBe("There's already a tag called “Cliente”");
  });

  it('los números llegan como datos y salen como texto', () => {
    const error: CommandError = {
      code: 'tag_name_too_long',
      params: { max: 32, len: 40 },
      message: 'el nombre de la etiqueta no puede pasar de 32 caracteres (tiene 40)',
    };
    expect(errorText(error, 'en')).toBe("The tag name can't be longer than 32 characters (it has 40)");
  });

  it('traduce también la causa de una copia dañada', () => {
    const error: CommandError = {
      code: 'backup_damaged',
      params: {
        cause: {
          code: 'tag_name_empty',
          params: {},
          message: 'el nombre de la etiqueta no puede estar vacío',
        },
      },
      message: 'la copia está dañada: el nombre de la etiqueta no puede estar vacío',
    };
    expect(errorText(error, 'en')).toBe("The backup is damaged: the tag name can't be empty");
    expect(errorText(error, 'es')).toBe(error.message);
  });

  it('un código que no conoce sale con el mensaje del backend', () => {
    const error: CommandError = { code: 'nuevo_de_una_version_futura', params: {}, message: 'algo nuevo' };
    expect(errorText(error, 'en')).toBe('algo nuevo');
  });

  it('lo que no viene del backend sale tal cual', () => {
    expect(errorText('el diálogo se cerró', 'en')).toBe('el diálogo se cerró');
    expect(errorText(new Error('permission denied'), 'en')).toBe('permission denied');
    expect(isCommandError({ code: 'x' })).toBe(false);
    expect(isCommandError(null)).toBe(false);
  });
});

/** Ficheros `.rs` del backend, sin sus módulos de tests. */
function rustSources(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return rustSources(path);
    if (!name.endsWith('.rs')) return [];
    return [readFileSync(path, 'utf8').split('#[cfg(test)]')[0] ?? ''];
  });
}

/** Cada código que puede salir del backend, con los datos que lleva. */
function backendCodes(): Map<string, Set<string>> {
  const codes = new Map<string, Set<string>>([
    ['db', new Set(['detail'])],
    ['migration', new Set(['detail'])],
    ['io', new Set(['detail'])],
    ['git', new Set(['detail'])],
  ]);
  const constructor = /AppError::(?:validation|not_found|internal)\(\s*"(\w+)"|invalid_path\([^;]*?,\s*"(\w+)",/g;
  for (const source of rustSources('src-tauri/src')) {
    for (const match of source.matchAll(constructor)) {
      const code = match[1] ?? match[2] ?? '';
      // Los datos que se le añaden van detrás, hasta el final de la expresión
      // o hasta el siguiente error.
      const rest = source.slice((match.index ?? 0) + match[0].length);
      // Un `;` dentro de un mensaje no cierra la expresión.
      const blanked = rest.replace(/"(?:[^"\\]|\\[\s\S])*"/g, (s) => `"${' '.repeat(s.length - 2)}"`);
      const end = blanked.search(/;|AppError::/);
      const tail = end === -1 ? rest : rest.slice(0, end);
      const params = codes.get(code) ?? new Set<string>();
      for (const [, key] of tail.matchAll(/\.with\(\s*"(\w+)"/g)) params.add(key ?? '');
      if (match[2]) params.add('path');
      if (tail.includes('.because(')) params.add('cause');
      codes.set(code, params);
    }
  }
  return codes;
}

/** Los datos que usa un texto: los nombres que desestructura su función. */
function usedParams(text: unknown): string[] {
  if (typeof text !== 'function') return [];
  const match = /^\(\{([^}]*)\}/.exec(text.toString());
  return (match?.[1] ?? '').split(',').map((name) => name.trim()).filter(Boolean);
}

describe('códigos del backend', () => {
  const codes = backendCodes();
  const en = errorTexts.en ?? {};

  it('encuentra los códigos en el código Rust', () => {
    expect(codes.size).toBeGreaterThan(40);
    expect(codes.get('tag_name_taken')).toEqual(new Set(['name']));
    expect(codes.get('path_unreadable')).toEqual(new Set(['path', 'detail']));
    expect(codes.get('backup_damaged')).toEqual(new Set(['cause']));
  });

  it('cada código tiene texto en inglés, y no sobra ninguno', () => {
    expect(Object.keys(en).sort()).toEqual([...codes.keys()].sort());
  });

  it('ningún texto en inglés espera un dato que el backend no manda', () => {
    for (const [code, params] of codes) {
      for (const name of usedParams(en[code])) {
        expect(params, `${code} usa «${name}»`).toContain(name);
      }
    }
  });
});
