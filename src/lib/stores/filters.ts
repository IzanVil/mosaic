import { derived, get, writable } from 'svelte/store';
import * as api from '../api/settings';
import type {
  FilterState,
  GitStateFilter,
  SortDirection,
  SortOption,
  ViewState,
} from '../types';
import { debounce } from '../utils/debounce';

export const DEFAULT_FILTERS: FilterState = {
  query: '',
  tag_ids: [],
  languages: [],
  git_state: 'all',
  pinned_only: false,
  include_missing: true,
  sort: 'name',
  sort_dir: 'asc',
};
const PERSIST_DELAY_MS = 500;
const GIT_STATES: GitStateFilter[] = ['all', 'dirty', 'clean', 'no_repo'];
const SORT_OPTIONS: SortOption[] = ['name', 'last_opened', 'created', 'updated'];
export const filters = writable<FilterState>({ ...DEFAULT_FILTERS });
export const sidebarCollapsed = writable(false);
export const hasActiveFilters = derived(filters, (current) => activeFilterCount(current) > 0);

export function activeFilterCount(current: FilterState): number {
  return (
    (current.query.trim() === '' ? 0 : 1) +
    current.tag_ids.length +
    current.languages.length +
    (current.git_state === 'all' ? 0 : 1) +
    (current.pinned_only ? 1 : 0) +
    (current.include_missing ? 0 : 1)
  );
}
export function setQuery(query: string): void {
  filters.update((current) => ({ ...current, query }));
}
export function toggleTagFilter(tagId: number): void {
  filters.update((current) => ({
    ...current,
    tag_ids: current.tag_ids.includes(tagId)
      ? current.tag_ids.filter((id) => id !== tagId)
      : [...current.tag_ids, tagId],
  }));
}

export function removeTagFilter(tagId: number): void {
  filters.update((current) =>
    current.tag_ids.includes(tagId)
      ? { ...current, tag_ids: current.tag_ids.filter((id) => id !== tagId) }
      : current,
  );
}
export function toggleLanguageFilter(language: string): void {
  filters.update((current) => ({
    ...current,
    languages: current.languages.includes(language)
      ? current.languages.filter((item) => item !== language)
      : [...current.languages, language],
  }));
}
export function setGitState(git_state: GitStateFilter): void {
  filters.update((current) => ({ ...current, git_state }));
}
export function setPinnedOnly(pinned_only: boolean): void {
  filters.update((current) => ({ ...current, pinned_only }));
}
export function setIncludeMissing(include_missing: boolean): void {
  filters.update((current) => ({ ...current, include_missing }));
}
export function setSort(sort: SortOption): void {
  filters.update((current) =>
    current.sort === sort
      ? { ...current, sort_dir: current.sort_dir === 'asc' ? 'desc' : 'asc' }
      : { ...current, sort, sort_dir: defaultDirectionFor(sort) },
  );
}
export function setSortDirection(sort_dir: SortDirection): void {
  filters.update((current) => ({ ...current, sort_dir }));
}
export function clearFilters(): void {
  filters.update((current) => ({
    ...DEFAULT_FILTERS,
    sort: current.sort,
    sort_dir: current.sort_dir,
  }));
}
export function toggleSidebar(): void {
  sidebarCollapsed.update((collapsed) => !collapsed);
}
function defaultDirectionFor(sort: SortOption): SortDirection {
  return sort === 'name' ? 'asc' : 'desc';
}
export async function loadViewState(): Promise<void> {
  let raw: string | null;
  try {
    raw = await api.getViewState();
  } catch {
    return;
  }
  if (raw === null) return;
  try {
    const parsed = sanitizeViewState(JSON.parse(raw));
    filters.set(parsed.filters);
    sidebarCollapsed.set(parsed.sidebar_collapsed);
  } catch {
  }
}
export function startPersistingViewState(): () => void {
  const save = debounce(() => {
    const state: ViewState = {
      filters: get(filters),
      sidebar_collapsed: get(sidebarCollapsed),
    };
    void api.setViewState(JSON.stringify(state)).catch(() => {
    });
  }, PERSIST_DELAY_MS);
  let pendingInitial = 2;
  const onChange = () => {
    if (pendingInitial > 0) {
      pendingInitial -= 1;
      return;
    }
    save();
  };
  const unsubscribes = [filters.subscribe(onChange), sidebarCollapsed.subscribe(onChange)];
  return () => {
    save.cancel();
    unsubscribes.forEach((unsubscribe) => unsubscribe());
  };
}
function sanitizeViewState(raw: unknown): ViewState {
  const source = isRecord(raw) ? raw : {};
  const stored = isRecord(source.filters) ? source.filters : {};
  return {
    filters: {
      query: typeof stored.query === 'string' ? stored.query : DEFAULT_FILTERS.query,
      tag_ids: numberArray(stored.tag_ids),
      languages: stringArray(stored.languages),
      git_state: oneOf(stored.git_state, GIT_STATES, DEFAULT_FILTERS.git_state),
      pinned_only: typeof stored.pinned_only === 'boolean' ? stored.pinned_only : false,
      include_missing:
        typeof stored.include_missing === 'boolean'
          ? stored.include_missing
          : DEFAULT_FILTERS.include_missing,
      sort: oneOf(stored.sort, SORT_OPTIONS, DEFAULT_FILTERS.sort),
      sort_dir: stored.sort_dir === 'desc' ? 'desc' : 'asc',
    },
    sidebar_collapsed: source.sidebar_collapsed === true,
  };
}
function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
function numberArray(value: unknown): number[] {
  return Array.isArray(value) ? value.filter((item): item is number => Number.isInteger(item)) : [];
}
function stringArray(value: unknown): string[] {
  return Array.isArray(value) ? value.filter((item): item is string => typeof item === 'string') : [];
}
function oneOf<T extends string>(value: unknown, allowed: T[], fallback: T): T {
  return allowed.includes(value as T) ? (value as T) : fallback;
}
