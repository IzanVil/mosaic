/** Wrappers de los comandos Tauri de proyectos. */

import { invoke } from '@tauri-apps/api/core';

import type { Project, ProjectWithTags, ReadmePreview } from '../types';

/** Devuelve todos los proyectos: primero los fijados, luego por nombre. */
export function listProjects(): Promise<Project[]> {
  return invoke<Project[]>('list_projects');
}

/**
 * Devuelve todos los proyectos con sus etiquetas y su estado Git cacheado.
 *
 * Es la lectura que alimenta el tablero: trae en una llamada todo lo que
 * necesita una tarjeta.
 */
export function listProjectsWithTags(): Promise<ProjectWithTags[]> {
  return invoke<ProjectWithTags[]>('list_projects_with_tags');
}

/** Devuelve un proyecto concreto. */
export function getProject(id: number): Promise<Project> {
  return invoke<Project>('get_project', { id });
}

/** Fija o quita la marca de favorito de un proyecto. */
export function setProjectPinned(id: number, pinned: boolean): Promise<void> {
  return invoke<void>('set_project_pinned', { id, pinned });
}

/**
 * Lee el README del proyecto, ya renderizado y saneado en el backend.
 * `null` si no tiene README o si la carpeta ya no está en disco.
 */
export function getProjectReadme(projectId: number): Promise<ReadmePreview | null> {
  return invoke<ReadmePreview | null>('get_project_readme', { projectId });
}

/**
 * Guarda las notas del proyecto y devuelve lo que ha quedado guardado: `null`
 * si el texto estaba vacío o era solo espacios.
 */
export function setProjectNotes(id: number, notes: string): Promise<string | null> {
  return invoke<string | null>('set_project_notes', { id, notes });
}
