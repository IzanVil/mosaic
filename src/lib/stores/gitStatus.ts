/** Refresco del estado Git y lo que pasó en el último; el estado vive en `stores/projects.ts`. */

import { writable } from 'svelte/store';

import * as api from '../api/git';
import type { GitRefreshSummary } from '../types';
import { errorText } from '../utils/errors';
import { now_ts } from '../utils/format';
import { mergeGitStatus, patchGitStatus } from './projects';

/** Qué pasó en el último refresco, para poder contarlo en la interfaz. */
export interface GitRefreshOutcome {
  /** Cuándo terminó, en segundos desde el epoch Unix. */
  at: number;
  /** Repositorios leídos. */
  read: number;
  /** Repositorios que no se pudieron leer. */
  failed: number;
  /** Proyectos cuyo estado cambió de verdad respecto a lo que había en pantalla. */
  changed: number;
  /** `true` si lo pidió el usuario, `false` si fue el refresco automático. */
  manual: boolean;
}

/** Duración mínima del indicador: leer tres repositorios tarda 40 ms y el clic no se veía. */
const MIN_SPINNER_MS = 450;

/** Hay un refresco pedido por el usuario en marcha. */
export const refreshingGit = writable(false);
/** Último error del refresco, para enseñarlo en la interfaz. */
export const gitError = writable<string | null>(null);
/** Lo que pasó en el último refresco. */
export const lastGitRefresh = writable<GitRefreshOutcome | null>(null);

function delay(milliseconds: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

/** Lee la caché tal y como está, sin releer los repositorios. La llama el arranque. */
export async function loadGitStatus(): Promise<void> {
  gitError.set(null);
  try {
    const entries = await api.listGitStatus();
    const changed = mergeGitStatus(entries);
    lastGitRefresh.set({
      at: now_ts(),
      read: entries.length,
      failed: 0,
      changed,
      manual: false,
    });
  } catch (error) {
    gitError.set(errorText(error));
  }
}

/** Aplica lo que trae un refresco, del botón o del evento automático (`manual` en `false`). */
export function applyGitRefresh(summary: GitRefreshSummary, manual = false): void {
  const changed = patchGitStatus(summary.changed, summary.removed);
  lastGitRefresh.set({
    at: now_ts(),
    read: summary.refreshed,
    failed: summary.failed,
    changed,
    manual,
  });
}

/** Relee todos los repositorios del disco y mezcla el resultado. */
export async function refreshAllGitStatus(): Promise<void> {
  refreshingGit.set(true);
  gitError.set(null);
  const started = Date.now();
  try {
    applyGitRefresh(await api.refreshAllGitStatus(), true);
  } catch (error) {
    gitError.set(errorText(error));
  } finally {
    const elapsed = Date.now() - started;
    if (elapsed < MIN_SPINNER_MS) await delay(MIN_SPINNER_MS - elapsed);
    refreshingGit.set(false);
  }
}
