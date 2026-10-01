/**
 * El número del botón «Filtros» tiene que coincidir con los chips a la vista.
 *
 * Es el problema 7 de la auditoría: la búsqueda contaba como filtro pero no se
 * pintaba como chip, y el botón decía «Filtros 4» con tres chips debajo. La
 * búsqueda sigue recortando la lista, así que `hasActiveFilters` sí la cuenta.
 */

import { get } from 'svelte/store';
import { beforeEach, describe, expect, it } from 'vitest';

import {
  DEFAULT_FILTERS,
  filterChipCount,
  filters,
  hasActiveFilters,
} from '../../src/lib/stores/filters';
import type { FilterState } from '../../src/lib/types';

beforeEach(() => {
  filters.set({ ...DEFAULT_FILTERS });
});

describe('contador de chips', () => {
  it('no cuenta la búsqueda', () => {
    expect(filterChipCount({ ...DEFAULT_FILTERS, query: 'mosaic' })).toBe(0);
  });

  it('cuenta un chip por cada filtro que la barra pinta', () => {
    const state: FilterState = {
      ...DEFAULT_FILTERS,
      query: 'mosaic',
      tag_ids: [1, 2],
      languages: ['Rust'],
      git_state: 'dirty',
      pinned_only: true,
      include_missing: false,
    };
    expect(filterChipCount(state)).toBe(6);
  });

  it('no cuenta la ordenación', () => {
    expect(filterChipCount({ ...DEFAULT_FILTERS, sort: 'created', sort_dir: 'desc' })).toBe(0);
  });
});

describe('hay filtros activos', () => {
  it('es falso con la vista por defecto', () => {
    expect(get(hasActiveFilters)).toBe(false);
  });

  it('es verdadero con solo una búsqueda, aunque no haya chips', () => {
    filters.set({ ...DEFAULT_FILTERS, query: 'mosaic' });
    expect(get(hasActiveFilters)).toBe(true);
  });

  it('ignora una búsqueda hecha solo de espacios', () => {
    filters.set({ ...DEFAULT_FILTERS, query: '   ' });
    expect(get(hasActiveFilters)).toBe(false);
  });

  it('es verdadero con un chip y sin búsqueda', () => {
    filters.set({ ...DEFAULT_FILTERS, languages: ['Go'] });
    expect(get(hasActiveFilters)).toBe(true);
  });
});
