import { describe, expect, it } from 'vitest';

import { opensUpward } from '../../src/lib/utils/placement';

describe('opensUpward', () => {
  it('abre abajo si cabe', () => {
    expect(opensUpward({ top: 100, bottom: 120 }, 300, 800)).toBe(false);
  });

  it('abre arriba si abajo no cabe y arriba sí', () => {
    expect(opensUpward({ top: 700, bottom: 720 }, 300, 800)).toBe(true);
  });

  it('abre abajo si no cabe en ningún lado', () => {
    expect(opensUpward({ top: 150, bottom: 170 }, 700, 800)).toBe(false);
  });

  it('cuenta el hueco entre disparador y panel', () => {
    expect(opensUpward({ top: 400, bottom: 496 }, 300, 800, 4)).toBe(false);
    expect(opensUpward({ top: 400, bottom: 497 }, 300, 800, 4)).toBe(true);
  });
});
