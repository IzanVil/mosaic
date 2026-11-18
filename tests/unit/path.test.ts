import { describe, expect, it } from 'vitest';

import { parentFolder } from '../../src/lib/utils/path';

describe('parentFolder', () => {
  it('abrevia la carpeta personal en Linux, macOS y Windows', () => {
    expect(parentFolder('/home/izan/proyectos/mosaic')).toBe('~/proyectos');
    expect(parentFolder('/Users/izan/code/mosaic')).toBe('~/code');
    expect(parentFolder('C:\\Users\\Izan\\code\\mosaic')).toBe('~\\code');
  });

  it('deja «~» a secas si el proyecto está directamente en la carpeta personal', () => {
    expect(parentFolder('/home/izan/mosaic')).toBe('~');
    expect(parentFolder('C:\\Users\\Izan\\mosaic')).toBe('~');
  });

  it('no abrevia lo que solo se parece a la carpeta personal', () => {
    expect(parentFolder('/home2/izan/mosaic')).toBe('/home2/izan');
    expect(parentFolder('/srv/home/izan/mosaic')).toBe('/srv/home/izan');
    expect(parentFolder('/homero/mosaic')).toBe('/homero');
  });

  it('conserva la raíz con su separador', () => {
    expect(parentFolder('/mosaic')).toBe('/');
    expect(parentFolder('C:\\mosaic')).toBe('C:\\');
  });

  it('ignora el separador final', () => {
    expect(parentFolder('/home/izan/proyectos/mosaic/')).toBe('~/proyectos');
    expect(parentFolder('D:\\trabajo\\mosaic\\')).toBe('D:\\trabajo');
  });

  it('devuelve tal cual una ruta sin separadores', () => {
    expect(parentFolder('mosaic')).toBe('mosaic');
  });
});
