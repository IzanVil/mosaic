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
  cycleTheme,
  density,
  detailTipDismissed,
  filters,
  setGitState,
  sanitizeViewState,
  sidebarCollapsed,
  theme,
  toggleDensity,
  toggleSidebar,
  toggleTagFilter,
} from '../../src/lib/stores/filters';

beforeEach(() => {
  filters.set({ ...DEFAULT_FILTERS });
  sidebarCollapsed.set(false);
  density.set('comodo');
  theme.set('dark');
  detailTipDismissed.set(false);
});

describe('lo que se guarda', () => {
  it('incluye todos los campos del estado de vista', () => {
    expect(Object.keys(currentViewState()).sort()).toEqual([
      'density',
      'detail_tip_dismissed',
      'filters',
      'sidebar_collapsed',
      'theme',
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

  it('cae al tema oscuro cuando falta el campo', () => {
    expect(sanitizeViewState({ density: 'compacto' }).theme).toBe('dark');
  });

  it('cae al tema oscuro con un valor inventado', () => {
    expect(sanitizeViewState({ theme: 'sepia' }).theme).toBe('dark');
  });

  it('acepta los tres temas guardados', () => {
    for (const mode of ['dark', 'light', 'system'] as const) {
      expect(sanitizeViewState({ theme: mode }).theme).toBe(mode);
    }
  });

  it('descarta un estado que no es un objeto sin perder los valores por defecto', () => {
    const recuperado = sanitizeViewState('esto no es una vista');
    expect(recuperado.filters).toEqual(DEFAULT_FILTERS);
    expect(recuperado.density).toBe('comodo');
    expect(recuperado.sidebar_collapsed).toBe(false);
  });
});

describe('el tema', () => {
  it('recorre oscuro, claro y sistema, y vuelve a empezar', () => {
    const vistos = [];
    for (let i = 0; i < 4; i += 1) {
      cycleTheme();
      vistos.push(currentViewState().theme);
    }
    expect(vistos).toEqual(['light', 'system', 'dark', 'light']);
  });

  it('sobrevive al viaje completo por JSON', () => {
    cycleTheme();
    cycleTheme();
    const recuperado = sanitizeViewState(JSON.parse(JSON.stringify(currentViewState())));
    expect(recuperado.theme).toBe('system');
  });
});

describe('el aviso de cómo abrir el detalle', () => {
  it('sale por defecto, también con una vista guardada por una versión anterior', () => {
    expect(sanitizeViewState({ density: 'compacto', theme: 'light' }).detail_tip_dismissed).toBe(
      false,
    );
  });

  it('una vez cerrado, sobrevive al viaje completo por JSON', () => {
    detailTipDismissed.set(true);
    const recuperado = sanitizeViewState(JSON.parse(JSON.stringify(currentViewState())));
    expect(recuperado.detail_tip_dismissed).toBe(true);
  });
});
