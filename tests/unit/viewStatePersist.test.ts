/**
 * Guardar la vista: cargarla no escribe, y el primer cambio de verdad sí.
 *
 * Al arrancar, cada store emite su valor una vez al suscribirse, y esas
 * emisiones no son cambios del usuario. El 2026-10-09 se quitó un store y el
 * número de emisiones a ignorar se quedó en el de antes: el primer cambio tras
 * abrir la app se perdía sin que fallara nada.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../../src/lib/api/settings', () => ({
  getViewState: vi.fn(async () => null),
  setViewState: vi.fn(async () => undefined),
}));

import { setViewState } from '../../src/lib/api/settings';
import { density, startPersistingViewState, toggleDensity } from '../../src/lib/stores/filters';

const guardar = vi.mocked(setViewState);
let parar: () => void = () => {};

beforeEach(() => {
  vi.useFakeTimers();
  guardar.mockClear();
  density.set('comodo');
});

afterEach(() => {
  parar();
  vi.useRealTimers();
});

describe('guardar la vista', () => {
  it('empezar a guardar no escribe nada', async () => {
    parar = startPersistingViewState();
    await vi.advanceTimersByTimeAsync(2000);
    expect(guardar).not.toHaveBeenCalled();
  });

  it('el primer cambio después de arrancar se guarda', async () => {
    parar = startPersistingViewState();
    toggleDensity();
    await vi.advanceTimersByTimeAsync(2000);
    expect(guardar).toHaveBeenCalledTimes(1);
    expect(JSON.parse(guardar.mock.calls[0]![0]).density).toBe('compacto');
  });
});
