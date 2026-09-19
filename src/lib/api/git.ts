/** Wrappers de los comandos Tauri de estado Git. */

import { invoke } from '@tauri-apps/api/core';

import type { GitRefreshSummary, GitStatusEntry } from '../types';

/** Devuelve toda la caché de estado Git. */
export function listGitStatus(): Promise<GitStatusEntry[]> {
  return invoke<GitStatusEntry[]>('list_git_status');
}

/**
 * Relee el estado Git de un proyecto.
 * Devuelve `null` si el proyecto no es un repositorio.
 */
export function refreshGitStatus(projectId: number): Promise<GitStatusEntry | null> {
  return invoke<GitStatusEntry | null>('refresh_git_status', { projectId });
}

/** Relee el estado Git de todos los proyectos que son repositorios. */
export function refreshAllGitStatus(): Promise<GitRefreshSummary> {
  return invoke<GitRefreshSummary>('refresh_all_git_status');
}
