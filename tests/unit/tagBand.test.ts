import { describe, expect, it } from 'vitest';

import { tagBand } from '../../src/lib/utils/tagColors';

describe('tagBand', () => {
  it('sin etiquetas no da cenefa: la pone la tarjeta', () => {
    expect(tagBand([])).toBeUndefined();
  });

  it('con una sola es su color, sin llagas', () => {
    expect(tagBand(['#3B82F6'])).toBe('#3B82F6');
  });

  it('reparte el ancho a partes iguales y deja la llaga al final de cada tramo menos el último', () => {
    expect(tagBand(['#3B82F6', '#F97316', '#EF4444'])).toBe(
      'linear-gradient(90deg, ' +
        '#3B82F6 0% calc(33.333% - var(--tile-grout)), transparent 0 33.333%, ' +
        '#F97316 33.333% calc(66.667% - var(--tile-grout)), transparent 0 66.667%, ' +
        '#EF4444 66.667% 100%)',
    );
  });

  it('respeta el orden de las etiquetas', () => {
    const band = tagBand(['#111111', '#222222']) ?? '';
    expect(band.indexOf('#111111')).toBeLessThan(band.indexOf('#222222'));
  });
});
