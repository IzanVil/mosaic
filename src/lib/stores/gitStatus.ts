/**
 * Refresco del estado Git y resultado del último intento.
 *
 * Este módulo ya no guarda una copia de la caché: la fuente de verdad del
 * tablero es `stores/projects.ts`, y aquí solo se orquesta la relectura y se
 * mezcla el resultado por `project_id`.
 */

import { writable } from 'svelte/store';

import * as api from '../api/git';
import { now_ts } from '../utils/format';
import { mergeGitStatus, mergeOneGitStatus } from './projects';

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

/**
 * Duración mínima del indicador de carga.
 *
 * Leer tres repositorios tarda 40 ms: sin este suelo, el usuario pulsa el botón
 * y no ve absolutamente nada, que es exactamente el bug que hubo que arreglar.
 */
const MIN_SPINNER_MS = 450;

export const refreshingGit = writable(false);
export const gitError = writable<string | null>(null);
export const lastGitRefresh = writable<GitRefreshOutcome | null>(null);

function delay(milliseconds: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

/**
 * Lee la caché tal y como está, sin releer los repositorios.
 *
 * La llama el arranque y el evento `git-status-refreshed` del backend, de ahí
 * que `manual` sea `false` por defecto.
 */
export async function loadGitStatus(manual = false): Promise<void> {
  gitError.set(null);
  try {
    const entries = await api.listGitStatus();
    const changed = mergeGitStatus(entries);
    lastGitRefresh.set({
      at: now_ts(),
      read: entries.length,
      failed: 0,
      changed,
      manual,
    });
  } catch (error) {
    gitError.set(String(error));
  }
}

/** Relee todos los repositorios del disco y mezcla el resultado. */
export async function refreshAllGitStatus(): Promise<void> {
  refreshingGit.set(true);
  gitError.set(null);
  const started = Date.now();
  try {
    const summary = await api.refreshAllGitStatus();
    const changed = mergeGitStatus(await api.listGitStatus());
    lastGitRefresh.set({
      at: now_ts(),
      read: summary.refreshed,
      failed: summary.failed,
      changed,
      manual: true,
    });
  } catch (error) {
    gitError.set(String(error));
  } finally {
    const elapsed = Date.now() - started;
    if (elapsed < MIN_SPINNER_MS) await delay(MIN_SPINNER_MS - elapsed);
    refreshingGit.set(false);
  }
}

/** Relee un único proyecto. Devuelve `null` en la caché si dejó de ser repositorio. */
export async function refreshOneGitStatus(projectId: number): Promise<void> {
  gitError.set(null);
  try {
    mergeOneGitStatus(projectId, await api.refreshGitStatus(projectId));
  } catch (error) {
    gitError.set(String(error));
  }
}
