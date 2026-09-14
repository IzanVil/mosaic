/** Estado global de la caché de Git, indexada por proyecto. */

import { writable } from 'svelte/store';

import * as api from '../api/git';
import type { GitStatusEntry } from '../types';

/** Estado Git por `project_id`. Un proyecto sin entrada no es repositorio. */
export const gitStatus = writable<Record<number, GitStatusEntry>>({});
export const refreshingGit = writable(false);
export const gitError = writable<string | null>(null);

function index(entries: GitStatusEntry[]): Record<number, GitStatusEntry> {
  return Object.fromEntries(entries.map((entry) => [entry.project_id, entry]));
}

/** Carga la caché tal y como está, sin releer los repositorios. */
export async function loadGitStatus(): Promise<void> {
  gitError.set(null);
  try {
    gitStatus.set(index(await api.listGitStatus()));
  } catch (error) {
    gitError.set(String(error));
  }
}

/** Relee todos los repositorios y actualiza la caché. */
export async function refreshAllGitStatus(): Promise<void> {
  refreshingGit.set(true);
  gitError.set(null);
  try {
    await api.refreshAllGitStatus();
    gitStatus.set(index(await api.listGitStatus()));
  } catch (error) {
    gitError.set(String(error));
  } finally {
    refreshingGit.set(false);
  }
}

/** Relee un único proyecto y actualiza su entrada. */
export async function refreshOneGitStatus(projectId: number): Promise<void> {
  gitError.set(null);
  try {
    const entry = await api.refreshGitStatus(projectId);
    gitStatus.update((current) => {
      const next = { ...current };
      if (entry === null) {
        delete next[projectId];
      } else {
        next[projectId] = entry;
      }
      return next;
    });
  } catch (error) {
    gitError.set(String(error));
  }
}
