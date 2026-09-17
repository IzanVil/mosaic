/** Wrappers de los comandos Tauri de proyectos. */

import { invoke } from '@tauri-apps/api/core';

import type { Project, ProjectWithTags } from '../types';

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
