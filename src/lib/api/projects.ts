/** Wrappers de los comandos Tauri de proyectos. */

import { invoke } from '@tauri-apps/api/core';

import type { Project } from '../types';

/** Devuelve todos los proyectos: primero los fijados, luego por nombre. */
export function listProjects(): Promise<Project[]> {
  return invoke<Project[]>('list_projects');
}

/** Devuelve un proyecto concreto. */
export function getProject(id: number): Promise<Project> {
  return invoke<Project>('get_project', { id });
}
