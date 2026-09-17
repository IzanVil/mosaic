import Fuse, { type IFuseOptions } from 'fuse.js';
import { derived, writable } from 'svelte/store';
import { listProjectsWithTags, setProjectPinned } from '../api/projects';
import { scanAllPaths } from '../api/scanner';
import type { FilterState, GitStatusEntry, ProjectWithTags, ScanSummary, Tag } from '../types';
import { compareNames, normalizeText } from '../utils/text';
import { filters } from './filters';
export const projects = writable<ProjectWithTags[]>([]);
export const loadingProjects = writable(false);
export const scanning = writable(false);
export const lastScan = writable<ScanSummary | null>(null);
export const projectsError = writable<string | null>(null);
const FUSE_OPTIONS: IFuseOptions<IndexedProject> = {
  keys: ['name', 'path'],
  threshold: 0.3,
  ignoreLocation: true,
};
interface IndexedProject {
  id: number;
  name: string;
  path: string;
}
let indexCache: { source: ProjectWithTags[]; fuse: Fuse<IndexedProject> } | null = null;
function searchIndex(source: ProjectWithTags[]): Fuse<IndexedProject> {
  if (indexCache?.source === source) return indexCache.fuse;
  const rows: IndexedProject[] = source.map((project) => ({
    id: project.id,
    name: normalizeText(project.name),
    path: normalizeText(project.path),
  }));
  const fuse = new Fuse(rows, FUSE_OPTIONS);
  indexCache = { source, fuse };
  return fuse;
}

function matchingIds(source: ProjectWithTags[], query: string): Set<number> | null {
  const needle = normalizeText(query.trim());
  if (needle === '') return null;

  const results = searchIndex(source).search(needle);
  return new Set(results.map((result) => result.item.id));
}
function matchesFilters(project: ProjectWithTags, current: FilterState): boolean {
  if (!current.include_missing && project.missing) return false;
  if (current.pinned_only && !project.pinned) return false;
  if (current.tag_ids.length > 0) {
    if (!project.tags.some((tag) => current.tag_ids.includes(tag.id))) return false;
  }
  if (current.languages.length > 0) {
    if (project.primary_language === null) return false;
    if (!current.languages.includes(project.primary_language)) return false;
  }

  switch (current.git_state) {
    case 'dirty':
      return project.git_status?.is_dirty === true;
    case 'clean':
      return project.is_git_repo && project.git_status?.is_dirty === false;
    case 'no_repo':
      return !project.is_git_repo;
    case 'all':
      return true;
  }
}

function compareBy(a: ProjectWithTags, b: ProjectWithTags, current: FilterState): number {
  switch (current.sort) {
    case 'name':
      return compareNames(a.name, b.name);
    case 'last_opened':
      return compareNullableNumbers(a.last_opened_at, b.last_opened_at);
    case 'created':
      return a.created_at - b.created_at;
    case 'updated':
      return a.updated_at - b.updated_at;
  }
}

function compareNullableNumbers(a: number | null, b: number | null): number {
  if (a === b) return 0;
  if (a === null) return 1;
  if (b === null) return -1;
  return a - b;
}
export const visibleProjects = derived([projects, filters], ([source, current]) => {
  const matches = matchingIds(source, current.query);
  const filtered = source.filter(
    (project) =>
      (matches === null || matches.has(project.id)) && matchesFilters(project, current),
  );
  const direction = current.sort_dir === 'asc' ? 1 : -1;
  return filtered.sort((a, b) => {
    if (a.pinned !== b.pinned) return a.pinned ? -1 : 1;
    const bySort = compareBy(a, b, current) * direction;
    return bySort !== 0 ? bySort : compareNames(a.name, b.name);
  });
});

export const availableLanguages = derived(projects, (source) => {
  const languages = new Set<string>();
  for (const project of source) {
    if (project.primary_language !== null) languages.add(project.primary_language);
  }
  return [...languages].sort(compareNames);
});

export const pinnedProjects = derived(projects, (source) =>
  source.filter((project) => project.pinned),
);

export async function loadProjects(): Promise<void> {
  loadingProjects.set(true);
  projectsError.set(null);
  try {
    projects.set(await listProjectsWithTags());
  } catch (error) {
    projectsError.set(String(error));
  } finally {
    loadingProjects.set(false);
  }
}
export async function runScan(): Promise<ScanSummary | null> {
  scanning.set(true);
  projectsError.set(null);
  try {
    const summary = await scanAllPaths();
    lastScan.set(summary);
    projects.set(await listProjectsWithTags());
    return summary;
  } catch (error) {
    projectsError.set(String(error));
    return null;
  } finally {
    scanning.set(false);
  }
}
export async function togglePinned(id: number, pinned: boolean): Promise<void> {
  patchProject(id, (project) => ({ ...project, pinned }));
  try {
    await setProjectPinned(id, pinned);
  } catch (error) {
    projectsError.set(String(error));
    await loadProjects();
  }
}
export function markProjectOpened(id: number, openedAt: number): void {
  patchProject(id, (project) => ({ ...project, last_opened_at: openedAt }));
}
export function mergeGitStatus(entries: GitStatusEntry[]): void {
  const byProject = new Map(entries.map((entry) => [entry.project_id, entry]));
  projects.update((current) =>
    current.map((project) => {
      const next = byProject.get(project.id) ?? null;
      return sameGitStatus(project.git_status, next) ? project : { ...project, git_status: next };
    }),
  );
}
export function mergeOneGitStatus(projectId: number, entry: GitStatusEntry | null): void {
  patchProject(projectId, (project) =>
    sameGitStatus(project.git_status, entry) ? project : { ...project, git_status: entry },
  );
}
export function attachTag(projectId: number, tag: Tag): void {
  patchProject(projectId, (project) =>
    project.tags.some((existing) => existing.id === tag.id)
      ? project
      : { ...project, tags: [...project.tags, tag].sort((a, b) => compareNames(a.name, b.name)) },
  );
}
export function detachTag(projectId: number, tagId: number): void {
  patchProject(projectId, (project) => ({
    ...project,
    tags: project.tags.filter((tag) => tag.id !== tagId),
  }));
}
export function applyTagChangeEverywhere(tagId: number, updated: Tag | null): void {
  projects.update((current) =>
    current.map((project) => {
      if (!project.tags.some((tag) => tag.id === tagId)) return project;
      return {
        ...project,
        tags:
          updated === null
            ? project.tags.filter((tag) => tag.id !== tagId)
            : project.tags
                .map((tag) => (tag.id === tagId ? updated : tag))
                .sort((a, b) => compareNames(a.name, b.name)),
      };
    }),
  );
}

function patchProject(
  id: number,
  change: (project: ProjectWithTags) => ProjectWithTags,
): void {
  projects.update((current) =>
    current.map((project) => (project.id === id ? change(project) : project)),
  );
}
function sameGitStatus(a: GitStatusEntry | null, b: GitStatusEntry | null): boolean {
  if (a === null || b === null) return a === b;
  return (
    a.refreshed_at === b.refreshed_at &&
    a.branch === b.branch &&
    a.ahead === b.ahead &&
    a.behind === b.behind &&
    a.is_dirty === b.is_dirty &&
    a.last_commit_sha === b.last_commit_sha &&
    a.last_commit_msg === b.last_commit_msg &&
    a.last_commit_at === b.last_commit_at &&
    a.remote_url === b.remote_url
  );
}
