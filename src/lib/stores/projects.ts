/** Estado global de los proyectos y del escaneo. */

import { writable } from 'svelte/store';

import { listProjects } from '../api/projects';
import { scanAllPaths } from '../api/scanner';
import type { Project, ScanSummary } from '../types';
import { refreshAllGitStatus } from './gitStatus';

export const projects = writable<Project[]>([]);
export const loadingProjects = writable(false);
export const scanning = writable(false);
export const lastScan = writable<ScanSummary | null>(null);
/** Último error de proyectos o de escaneo, para mostrarlo en la interfaz. */
export const projectsError = writable<string | null>(null);

/** Recarga la lista de proyectos desde la base de datos. */
export async function loadProjects(): Promise<void> {
  loadingProjects.set(true);
  projectsError.set(null);
  try {
    projects.set(await listProjects());
  } catch (error) {
    projectsError.set(String(error));
  } finally {
    loadingProjects.set(false);
  }
}

/**
 * Lanza un escaneo completo y refresca la lista al terminar.
 *
 * Devuelve el resumen, o `null` si el escaneo falló.
 */
export async function runScan(): Promise<ScanSummary | null> {
  scanning.set(true);
  projectsError.set(null);
  try {
    const summary = await scanAllPaths();
    lastScan.set(summary);
    projects.set(await listProjects());
    // Los proyectos recién descubiertos no tienen estado Git cacheado: sin esto
    // sus indicadores estarían vacíos hasta el siguiente refresco automático.
    await refreshAllGitStatus();
    return summary;
  } catch (error) {
    projectsError.set(String(error));
    return null;
  } finally {
    scanning.set(false);
  }
}
