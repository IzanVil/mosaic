/**
 * Ajustes avanzados de escaneo y Git, tal y como se editan en Ajustes.
 *
 * Cada cambio se guarda en el acto, sin botón, y el campo que se cambió dice
 * «Guardado» a su lado durante un momento: un ajuste que se guarda sin que se
 * note parece que no se ha guardado (§10.4). Si el backend lo rechaza, el
 * campo vuelve a su valor anterior y enseña el motivo.
 */

import { get, writable } from 'svelte/store';

import { getAdvancedSettings, setAdvancedSettings } from '../api/settings';
import type { AdvancedSettings, AdvancedSettingsView } from '../types';

/** Lo que tarda en irse el «Guardado» de un campo. */
export const SAVED_VISIBLE_MS = 2000;

/** Ajuste que se puede editar por separado. */
export type AdvancedField = keyof AdvancedSettings;

/** Estado del último cambio de un campo. */
export type FieldStatus =
  | { state: 'saving' }
  | { state: 'saved' }
  | { state: 'error'; message: string };

/** Valores, valores de fábrica y límites. `null` hasta que se cargan. */
export const advancedSettings = writable<AdvancedSettingsView | null>(null);
/** Error al cargar, para enseñarlo en lugar de la sección. */
export const advancedSettingsError = writable<string | null>(null);
/** Estado por campo. Un campo sin entrada no tiene nada que contar. */
export const fieldStatus = writable<Partial<Record<AdvancedField, FieldStatus>>>({});

const clearTimers = new Map<AdvancedField, ReturnType<typeof setTimeout>>();

function setStatus(field: AdvancedField, status: FieldStatus | null): void {
  clearTimeout(clearTimers.get(field));
  fieldStatus.update((current) => {
    const next = { ...current };
    if (status === null) delete next[field];
    else next[field] = status;
    return next;
  });
  if (status?.state === 'saved') {
    clearTimers.set(
      field,
      setTimeout(() => setStatus(field, null), SAVED_VISIBLE_MS),
    );
  }
}

/** Lee los ajustes del backend. */
export async function loadAdvancedSettings(): Promise<void> {
  try {
    advancedSettings.set(await getAdvancedSettings());
    advancedSettingsError.set(null);
  } catch (error) {
    advancedSettingsError.set(String(error));
  }
}

/**
 * Cambia un ajuste y lo guarda ya.
 *
 * El valor nuevo se ve al momento; si el backend lo rechaza, se vuelve al
 * anterior y el campo dice por qué. Devuelve si se guardó.
 */
export async function saveAdvancedField<K extends AdvancedField>(
  field: K,
  value: AdvancedSettings[K],
): Promise<boolean> {
  const view = get(advancedSettings);
  if (view === null) return false;

  const previous = view.current;
  const next: AdvancedSettings = { ...previous, [field]: value };
  advancedSettings.set({ ...view, current: next });
  setStatus(field, { state: 'saving' });

  try {
    const stored = await setAdvancedSettings(next);
    advancedSettings.update((current) => (current ? { ...current, current: stored } : current));
    setStatus(field, { state: 'saved' });
    return true;
  } catch (error) {
    advancedSettings.update((current) => (current ? { ...current, current: previous } : current));
    setStatus(field, { state: 'error', message: String(error) });
    return false;
  }
}
