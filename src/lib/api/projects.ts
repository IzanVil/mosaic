/** Wrappers de los comandos Tauri de proyectos. */

import { invoke } from '@tauri-apps/api/core';

import type { ProjectWithTags, ReadmePreview } from '../types';

/** Todos los proyectos con sus etiquetas y su estado Git: el tablero, en una llamada. */
export function listProjectsWithTags(): Promise<ProjectWithTags[]> {
  return invoke<ProjectWithTags[]>('list_projects_with_tags');
}

/** Fija o quita la marca de favorito de un proyecto. */
export function setProjectPinned(id: number, pinned: boolean): Promise<void> {
  return invoke<void>('set_project_pinned', { id, pinned });
}

/** El README ya renderizado y saneado; `null` si no tiene o si la carpeta ya no está. */
export function getProjectReadme(projectId: number): Promise<ReadmePreview | null> {
  return invoke<ReadmePreview | null>('get_project_readme', { projectId });
}

/** Guarda las notas y devuelve lo que ha quedado: `null` si estaban en blanco. */
export function setProjectNotes(id: number, notes: string): Promise<string | null> {
  return invoke<string | null>('set_project_notes', { id, notes });
}
