/** Wrappers de los comandos Tauri de etiquetas. */

import { invoke } from '@tauri-apps/api/core';

import type { Tag, TagWithCount } from '../types';

/** Crea una etiqueta. El backend recorta el nombre y normaliza el color. */
export function createTag(name: string, color: string): Promise<Tag> {
  return invoke<Tag>('create_tag', { name, color });
}

/** Devuelve todas las etiquetas con su número de proyectos. */
export function listTags(): Promise<TagWithCount[]> {
  return invoke<TagWithCount[]>('list_tags');
}

/** Renombra una etiqueta y/o le cambia el color. */
export function updateTag(id: number, name: string, color: string): Promise<Tag> {
  return invoke<Tag>('update_tag', { id, name, color });
}

/** Borra una etiqueta y, por cascada, sus asignaciones. */
export function deleteTag(id: number): Promise<void> {
  return invoke<void>('delete_tag', { id });
}

/** Asigna una etiqueta a un proyecto. Repetirlo no falla. */
export function assignTag(projectId: number, tagId: number): Promise<void> {
  return invoke<void>('assign_tag', { projectId, tagId });
}

/** Quita una etiqueta de un proyecto. Quitar algo no asignado no falla. */
export function unassignTag(projectId: number, tagId: number): Promise<void> {
  return invoke<void>('unassign_tag', { projectId, tagId });
}

/** Devuelve las etiquetas de un proyecto, ordenadas por nombre. */
export function listTagsForProject(projectId: number): Promise<Tag[]> {
  return invoke<Tag[]>('list_tags_for_project', { projectId });
}
