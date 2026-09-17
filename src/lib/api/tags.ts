import { invoke } from '@tauri-apps/api/core';

import type { Tag, TagWithCount } from '../types';

export function createTag(name: string, color: string): Promise<Tag> {
  return invoke<Tag>('create_tag', { name, color });
}

export function listTags(): Promise<TagWithCount[]> {
  return invoke<TagWithCount[]>('list_tags');
}

export function updateTag(id: number, name: string, color: string): Promise<Tag> {
  return invoke<Tag>('update_tag', { id, name, color });
}

export function deleteTag(id: number): Promise<void> {
  return invoke<void>('delete_tag', { id });
}

export function assignTag(projectId: number, tagId: number): Promise<void> {
  return invoke<void>('assign_tag', { projectId, tagId });
}

export function unassignTag(projectId: number, tagId: number): Promise<void> {
  return invoke<void>('unassign_tag', { projectId, tagId });
}

export function listTagsForProject(projectId: number): Promise<Tag[]> {
  return invoke<Tag[]>('list_tags_for_project', { projectId });
}
