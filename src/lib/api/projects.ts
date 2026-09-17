import { invoke } from '@tauri-apps/api/core';

import type { Project, ProjectWithTags } from '../types';

export function listProjects(): Promise<Project[]> {
  return invoke<Project[]>('list_projects');
}

export function listProjectsWithTags(): Promise<ProjectWithTags[]> {
  return invoke<ProjectWithTags[]>('list_projects_with_tags');
}
export function getProject(id: number): Promise<Project> {
  return invoke<Project>('get_project', { id });
}

export function setProjectPinned(id: number, pinned: boolean): Promise<void> {
  return invoke<void>('set_project_pinned', { id, pinned });
}
