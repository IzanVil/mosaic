import { derived, get, writable } from 'svelte/store';

import * as api from '../api/tags';
import type { Tag, TagWithCount } from '../types';
import { compareNames } from '../utils/text';
import { removeTagFilter } from './filters';
import { applyTagChangeEverywhere, attachTag, detachTag } from './projects';

export const tags = writable<TagWithCount[]>([]);
export const loadingTags = writable(false);
export const tagsError = writable<string | null>(null);

export const tagsById = derived(tags, (current) => new Map(current.map((tag) => [tag.id, tag])));

export async function loadTags(): Promise<void> {
  loadingTags.set(true);
  tagsError.set(null);
  try {
    tags.set(await api.listTags());
  } catch (error) {
    tagsError.set(String(error));
  } finally {
    loadingTags.set(false);
  }
}

export async function createTag(name: string, color: string): Promise<Tag | null> {
  tagsError.set(null);
  try {
    const created = await api.createTag(name, color);
    tags.update((current) =>
      [...current, { ...created, project_count: 0 }].sort((a, b) => compareNames(a.name, b.name)),
    );
    return created;
  } catch (error) {
    tagsError.set(String(error));
    return null;
  }
}

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
    tagsError.set(String(error));
    return null;
  }
}

export async function deleteTag(id: number): Promise<boolean> {
  tagsError.set(null);
  try {
    await api.deleteTag(id);
    tags.update((current) => current.filter((tag) => tag.id !== id));
    applyTagChangeEverywhere(id, null);
    removeTagFilter(id);
    return true;
  } catch (error) {
    tagsError.set(String(error));
    return false;
  }
}

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
    tagsError.set(String(error));
    return false;
  }
}

export async function unassignTagFromProject(projectId: number, tagId: number): Promise<boolean> {
  tagsError.set(null);
  try {
    await api.unassignTag(projectId, tagId);
    detachTag(projectId, tagId);
    bumpCount(tagId, -1);
    return true;
  } catch (error) {
    tagsError.set(String(error));
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
