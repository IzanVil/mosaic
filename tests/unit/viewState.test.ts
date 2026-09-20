/**
 * La vista guardada tiene que sobrevivir al viaje de ida y vuelta completo.
 *
 * En la Fase 4 este mecanismo ya se rompió una vez: se añadió un campo al
 * estado y el guardado no lo incluía, así que la preferencia se perdía entre
 * sesiones sin que fallara nada. Estos tests cubren esa clase de error.
 */

import { describe, expect, it, beforeEach } from 'vitest';

import {
  DEFAULT_FILTERS,
  currentViewState,
  density,
  filters,
  setGitState,
  sanitizeViewState,
  sidebarCollapsed,
  toggleDensity,
  toggleSidebar,
  toggleTagFilter,
} from '../../src/lib/stores/filters';

beforeEach(() => {
  filters.set({ ...DEFAULT_FILTERS });
  sidebarCollapsed.set(false);
  density.set('comodo');
});

describe('lo que se guarda', () => {
  it('incluye todos los campos del estado de vista', () => {
    expect(Object.keys(currentViewState()).sort()).toEqual([
      'density',
      'filters',
      'sidebar_collapsed',
    ]);
  });

  it('recoge la densidad después de cambiarla', () => {
    toggleDensity();
    expect(currentViewState().density).toBe('compacto');
  });

  it('recoge el resto de preferencias de la vista', () => {
    toggleSidebar();
    toggleTagFilter(7);
    setGitState('dirty');

    const guardado = currentViewState();
    expect(guardado.sidebar_collapsed).toBe(true);
    expect(guardado.filters.tag_ids).toEqual([7]);
    expect(guardado.filters.git_state).toBe('dirty');
  });
});

describe('lo que se recupera', () => {
  it('sobrevive al viaje completo por JSON', () => {
    toggleDensity();
    toggleSidebar();
    toggleTagFilter(3);

    const recuperado = sanitizeViewState(JSON.parse(JSON.stringify(currentViewState())));

    expect(recuperado).toEqual(currentViewState());
  });

  it('cae a la densidad cómoda cuando falta el campo', () => {
    expect(sanitizeViewState({ filters: {}, sidebar_collapsed: false }).density).toBe('comodo');
  });

  it('cae a la densidad cómoda con un valor inventado', () => {
    expect(sanitizeViewState({ density: 'enorme' }).density).toBe('comodo');
  });

  it('acepta la densidad compacta guardada', () => {
    expect(sanitizeViewState({ density: 'compacto' }).density).toBe('compacto');
  });

  it('descarta un estado que no es un objeto sin perder los valores por defecto', () => {
    const recuperado = sanitizeViewState('esto no es una vista');
    expect(recuperado.filters).toEqual(DEFAULT_FILTERS);
    expect(recuperado.density).toBe('comodo');
    expect(recuperado.sidebar_collapsed).toBe(false);
  });
});
