/**
 * El guardado automático de los ajustes avanzados: se guarda al cambiar, dice
 * «Guardado» un momento, y si el backend lo rechaza vuelve al valor anterior.
 */

import { get } from 'svelte/store';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const defaults = {
  max_depth: 4,
  excluded_dirs: ['node_modules'],
  max_entries_per_scan: 50_000,
  scan_on_startup: false,
  git_refresh_interval_minutes: 5,
};

vi.mock('../../src/lib/api/settings', () => ({
  getAdvancedSettings: vi.fn(async () => ({
    current: { ...defaults },
    defaults: { ...defaults },
    limits: {
      max_depth_min: 1,
      max_depth_max: 8,
      max_entries_min: 1000,
      max_entries_max: 500000,
      git_refresh_max_minutes: 60,
      excluded_dirs_max: 100,
      excluded_dir_max_chars: 255,
    },
  })),
  setAdvancedSettings: vi.fn(async (settings: typeof defaults) => {
    if (settings.max_depth > 8) throw 'la profundidad tiene que estar entre 1 y 8';
    return { ...settings, excluded_dirs: settings.excluded_dirs.map((d) => d.trim()) };
  }),
}));

import { setAdvancedSettings } from '../../src/lib/api/settings';
import {
  SAVED_VISIBLE_MS,
  advancedSettings,
  fieldStatus,
  loadAdvancedSettings,
  saveAdvancedField,
} from '../../src/lib/stores/advancedSettings';

beforeEach(async () => {
  vi.useFakeTimers();
  vi.mocked(setAdvancedSettings).mockClear();
  await loadAdvancedSettings();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('ajustes avanzados', () => {
  it('guardan al cambiar y dicen «Guardado» un momento', async () => {
    expect(await saveAdvancedField('max_depth', 6)).toBe(true);
    expect(setAdvancedSettings).toHaveBeenCalledWith({ ...defaults, max_depth: 6 });
    expect(get(advancedSettings)?.current.max_depth).toBe(6);
    expect(get(fieldStatus).max_depth).toEqual({ state: 'saved' });

    await vi.advanceTimersByTimeAsync(SAVED_VISIBLE_MS);
    expect(get(fieldStatus).max_depth).toBeUndefined();
  });

  it('se quedan con lo que guardó el backend, ya normalizado', async () => {
    await saveAdvancedField('excluded_dirs', ['  build  ']);
    expect(get(advancedSettings)?.current.excluded_dirs).toEqual(['build']);
  });

  it('vuelven al valor anterior y dicen por qué si el backend lo rechaza', async () => {
    expect(await saveAdvancedField('max_depth', 12)).toBe(false);
    expect(get(advancedSettings)?.current.max_depth).toBe(4);
    expect(get(fieldStatus).max_depth).toEqual({
      state: 'error',
      message: 'la profundidad tiene que estar entre 1 y 8',
    });
  });
});
