/** Wrappers de los comandos Tauri de estado Git. */

import { invoke } from '@tauri-apps/api/core';

import type { Branches, CommitInfo, GitRefreshSummary, GitStatusEntry } from '../types';

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

/**
 * Devuelve los últimos commits del proyecto, del más reciente al más antiguo.
 * Vacío si el proyecto no es un repositorio o ya no está en disco.
 */
export function getProjectHistory(projectId: number): Promise<CommitInfo[]> {
  return invoke<CommitInfo[]>('get_project_history', { projectId });
}

/** Devuelve las ramas locales y las remotas conocidas del proyecto. */
export function getProjectBranches(projectId: number): Promise<Branches> {
  return invoke<Branches>('get_project_branches', { projectId });
}
