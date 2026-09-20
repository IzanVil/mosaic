/**
 * Búsqueda, filtros, ordenación y estado del sidebar.
 *
 * Este módulo solo guarda la *intención* del usuario; quien la aplica es el
 * store derivado `visibleProjects` de `stores/projects.ts`. La separación evita
 * el bucle clásico de escribir filtros desde un efecto que lee la lista ya
 * filtrada: aquí todo se cambia desde manejadores de eventos.
 *
 * La vista se persiste en la clave `ui.view_state` con 500 ms de debounce, no
 * en cada tecla.
 */

import { derived, get, writable } from 'svelte/store';

import * as api from '../api/settings';
import type {
  Density,
  FilterState,
  GitStateFilter,
  SortDirection,
  SortOption,
  ViewState,
} from '../types';
import { debounce } from '../utils/debounce';

/** Vista inicial de una instalación nueva. */
export const DEFAULT_FILTERS: FilterState = {
  query: '',
  tag_ids: [],
  languages: [],
  git_state: 'all',
  pinned_only: false,
  // Los proyectos ausentes se muestran por defecto: son los que el usuario
  // querrá revisar o soltar, esconderlos por su cuenta sería perderlos.
  include_missing: true,
  sort: 'name',
  sort_dir: 'asc',
};

const PERSIST_DELAY_MS = 500;

const GIT_STATES: GitStateFilter[] = ['all', 'dirty', 'clean', 'no_repo'];
const SORT_OPTIONS: SortOption[] = ['name', 'last_opened', 'created', 'updated'];

export const filters = writable<FilterState>({ ...DEFAULT_FILTERS });
export const sidebarCollapsed = writable(false);

/**
 * Densidad del tablero.
 *
 * Arranca en cómodo: quien tiene veinte proyectos no necesita apretarlos, y
 * quien tiene trescientos descubre el modo compacto en cuanto le estorba el
 * scroll.
 */
export const density = writable<Density>('comodo');

/** `true` cuando algo recorta la lista; el orden no cuenta como filtro. */
export const hasActiveFilters = derived(filters, (current) => activeFilterCount(current) > 0);

/** Cuántos filtros hay puestos, para el contador del botón «Filtros». */
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

/** Añade o quita una etiqueta de la selección (se combinan en OR). */
export function toggleTagFilter(tagId: number): void {
  filters.update((current) => ({
    ...current,
    tag_ids: current.tag_ids.includes(tagId)
      ? current.tag_ids.filter((id) => id !== tagId)
      : [...current.tag_ids, tagId],
  }));
}

/**
 * Saca una etiqueta de la selección.
 *
 * La llama el borrado de etiquetas: si el id borrado siguiera en los filtros,
 * el tablero aparecería vacío sin nada que lo explique la próxima vez que se
 * abriera la aplicación.
 */
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

/** Cambia el criterio de orden; repetir el mismo invierte la dirección. */
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

/** Quita todos los filtros y la búsqueda, conservando la ordenación. */
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

export function toggleDensity(): void {
  density.update((actual) => (actual === 'comodo' ? 'compacto' : 'comodo'));
}

/**
 * Dirección natural de cada criterio: los nombres se leen de la A a la Z, y
 * las fechas de lo más reciente a lo más antiguo.
 */
function defaultDirectionFor(sort: SortOption): SortDirection {
  return sort === 'name' ? 'asc' : 'desc';
}

/**
 * Lee la vista guardada. Hay que llamarla (y esperarla) antes del primer
 * render para no pintar el tablero sin filtros y reordenarlo un instante
 * después.
 *
 * Cualquier campo ausente o con un tipo inesperado cae a su valor por defecto:
 * una vista guardada por una versión anterior no debe romper el arranque.
 */
export async function loadViewState(): Promise<void> {
  let raw: string | null;
  try {
    raw = await api.getViewState();
  } catch {
    // La vista es una comodidad: si no se puede leer, se arranca con los
    // valores por defecto en silencio.
    return;
  }
  if (raw === null) return;

  try {
    const parsed = sanitizeViewState(JSON.parse(raw));
    filters.set(parsed.filters);
    sidebarCollapsed.set(parsed.sidebar_collapsed);
    density.set(parsed.density);
  } catch {
    /* JSON ilegible: se queda lo que hay. */
  }
}

/**
 * Empieza a guardar cada cambio de vista, con debounce.
 *
 * Devuelve la función de parada, que cancela la escritura pendiente. Hay que
 * llamarla después de [`loadViewState`]: la primera emisión de cada store se
 * ignora precisamente para que cargar no provoque una escritura.
 */
export function startPersistingViewState(): () => void {
  const save = debounce(() => {
    const state: ViewState = {
      filters: get(filters),
      sidebar_collapsed: get(sidebarCollapsed),
      density: get(density),
    };
    void api.setViewState(JSON.stringify(state)).catch(() => {
      /* Perder la vista guardada no debe interrumpir al usuario. */
    });
  }, PERSIST_DELAY_MS);

  let pendingInitial = 3;
  const onChange = () => {
    if (pendingInitial > 0) {
      pendingInitial -= 1;
      return;
    }
    save();
  };

  const unsubscribes = [
    filters.subscribe(onChange),
    sidebarCollapsed.subscribe(onChange),
    density.subscribe(onChange),
  ];

  return () => {
    save.cancel();
    unsubscribes.forEach((unsubscribe) => unsubscribe());
  };
}

/** Reconstruye un `ViewState` válido a partir de lo que hubiera guardado. */
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
    density: source.density === 'compacto' ? 'compacto' : 'comodo',
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
