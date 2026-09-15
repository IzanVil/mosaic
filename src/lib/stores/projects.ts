/** Estado global de los proyectos y del escaneo. */

import { writable } from 'svelte/store';

import { listProjects, setProjectPinned } from '../api/projects';
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

/**
 * Fija o deja de fijar un proyecto.
 *
 * Actualiza la lista en local antes de esperar al backend para que la tarjeta
 * responda al instante, y recarga si la escritura falla.
 */
export async function togglePinned(id: number, pinned: boolean): Promise<void> {
  projects.update((current) =>
    current.map((project) => (project.id === id ? { ...project, pinned } : project)),
  );

  try {
    await setProjectPinned(id, pinned);
    // El orden depende de `pinned`, así que se relee para reordenar la rejilla.
    projects.set(await listProjects());
  } catch (error) {
    projectsError.set(String(error));
    await loadProjects();
  }
}
