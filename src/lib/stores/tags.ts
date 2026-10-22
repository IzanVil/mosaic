import { derived, get, writable } from 'svelte/store';

import * as api from '../api/tags';
import type { Tag, TagWithCount } from '../types';
import { errorText } from '../utils/errors';
import { compareNames } from '../utils/text';
import { removeTagFilter } from './filters';
import { applyTagChangeEverywhere, attachTag, detachTag } from './projects';

/** Todas las etiquetas con su número de proyectos, ordenadas por nombre. */
export const tags = writable<TagWithCount[]>([]);
/** Las etiquetas se están cargando. */
export const loadingTags = writable(false);
/** Último error de etiquetas, para enseñarlo en la interfaz. */
export const tagsError = writable<string | null>(null);

/** Las mismas etiquetas, por id. */
export const tagsById = derived(tags, (current) => new Map(current.map((tag) => [tag.id, tag])));

/** Recarga las etiquetas desde la base de datos. */
export async function loadTags(): Promise<void> {
  loadingTags.set(true);
  tagsError.set(null);
  try {
    tags.set(await api.listTags());
  } catch (error) {
    tagsError.set(errorText(error));
  } finally {
    loadingTags.set(false);
  }
}

/** Crea una etiqueta; `null` si el backend la rechaza. */
export async function createTag(name: string, color: string): Promise<Tag | null> {
  tagsError.set(null);
  try {
    const created = await api.createTag(name, color);
    tags.update((current) =>
      [...current, { ...created, project_count: 0 }].sort((a, b) => compareNames(a.name, b.name)),
    );
    return created;
  } catch (error) {
    tagsError.set(errorText(error));
    return null;
  }
}

/** Renombra o cambia el color de una etiqueta, también en las tarjetas que la llevan. */
export async function updateTag(id: number, name: string, color: string): Promise<Tag | null> {
  tagsError.set(null);
  try {
    const updated = await api.updateTag(id, name, color);
    tags.update((current) =>
      current
        .map((tag) => (tag.id === id ? { ...updated, project_count: tag.project_count } : tag))
        .sort((a, b) => compareNames(a.name, b.name)),
    );
    applyTagChangeEverywhere(id, updated);
    return updated;
  } catch (error) {
    tagsError.set(errorText(error));
    return null;
  }
}

/** Borra una etiqueta, la quita de las tarjetas y de los filtros. */
export async function deleteTag(id: number): Promise<boolean> {
  tagsError.set(null);
  try {
    await api.deleteTag(id);
    tags.update((current) => current.filter((tag) => tag.id !== id));
    applyTagChangeEverywhere(id, null);
    removeTagFilter(id);
    return true;
  } catch (error) {
    tagsError.set(errorText(error));
    return false;
  }
}

/** Asigna una etiqueta a un proyecto y ajusta su conteo, sin recalcularlo. */
export async function assignTagToProject(projectId: number, tagId: number): Promise<boolean> {
  const tag = get(tagsById).get(tagId);
  if (tag === undefined) return false;

  tagsError.set(null);
  try {
    await api.assignTag(projectId, tagId);
    attachTag(projectId, tag);
    bumpCount(tagId, 1);
    return true;
  } catch (error) {
    tagsError.set(errorText(error));
    return false;
  }
}

/** Quita una etiqueta de un proyecto y ajusta su conteo. */
export async function unassignTagFromProject(projectId: number, tagId: number): Promise<boolean> {
  tagsError.set(null);
  try {
    await api.unassignTag(projectId, tagId);
    detachTag(projectId, tagId);
    bumpCount(tagId, -1);
    return true;
  } catch (error) {
    tagsError.set(errorText(error));
    return false;
  }
}

function bumpCount(tagId: number, delta: number): void {
  tags.update((current) =>
    current.map((tag) =>
      tag.id === tagId
        ? { ...tag, project_count: Math.max(0, tag.project_count + delta) }
        : tag,
    ),
  );
}
