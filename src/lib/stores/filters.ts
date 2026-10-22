/**
 * Búsqueda, filtros, orden y panel lateral: solo la intención del usuario.
 *
 * La aplica `visibleProjects`, y aquí todo se cambia desde manejadores de
 * eventos: escribir filtros desde un efecto que lee la lista filtrada es el
 * bucle clásico de Svelte. La vista se guarda en `ui.view_state`.
 */

import { derived, get, writable } from 'svelte/store';

import * as api from '../api/settings';
import { LANG_PREFERENCES, langPreference } from '../i18n';
import type {
  Density,
  FilterState,
  GitStateFilter,
  SortDirection,
  SortOption,
  ThemeMode,
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
  // Los ausentes se ven por defecto: esconderlos sin que nadie lo pida sería perderlos.
  include_missing: true,
  sort: 'name',
  sort_dir: 'asc',
};

const PERSIST_DELAY_MS = 500;

const GIT_STATES: GitStateFilter[] = ['all', 'dirty', 'clean', 'no_repo'];
const SORT_OPTIONS: SortOption[] = ['name', 'last_opened', 'created', 'updated'];
const THEMES: ThemeMode[] = ['dark', 'light', 'system'];

/** Búsqueda, filtros y orden que ha elegido el usuario. */
export const filters = writable<FilterState>({ ...DEFAULT_FILTERS });
/** El panel lateral está plegado. */
export const sidebarCollapsed = writable(false);

/** Densidad del tablero; arranca en cómodo, y quien tiene trescientos ya buscará el compacto. */
export const density = writable<Density>('comodo');

/** Tema de la interfaz; arranca en oscuro, el del producto, y `system` sigue al sistema. */
export const theme = writable<ThemeMode>('dark');

/** `true` si algo recorta la lista, búsqueda incluida; el orden no cuenta. */
export const hasActiveFilters = derived(
  filters,
  (current) => current.query.trim() !== '' || filterChipCount(current) > 0,
);

/** Chips de la barra de filtros, sin la búsqueda: si no, «Filtros 4» con tres chips a la vista. */
export function filterChipCount(current: FilterState): number {
  return (
    current.tag_ids.length +
    current.languages.length +
    (current.git_state === 'all' ? 0 : 1) +
    (current.pinned_only ? 1 : 0) +
    (current.include_missing ? 0 : 1)
  );
}

/** Cambia el texto de búsqueda. */
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

/** Saca una etiqueta borrada de la selección: si no, el tablero saldría vacío sin explicación. */
export function removeTagFilter(tagId: number): void {
  filters.update((current) =>
    current.tag_ids.includes(tagId)
      ? { ...current, tag_ids: current.tag_ids.filter((id) => id !== tagId) }
      : current,
  );
}

/** Añade o quita un lenguaje de la selección (se combinan en OR). */
export function toggleLanguageFilter(language: string): void {
  filters.update((current) => ({
    ...current,
    languages: current.languages.includes(language)
      ? current.languages.filter((item) => item !== language)
      : [...current.languages, language],
  }));
}

/** Filtra por estado de Git. */
export function setGitState(git_state: GitStateFilter): void {
  filters.update((current) => ({ ...current, git_state }));
}

/** Enseña solo los fijados, o todos. */
export function setPinnedOnly(pinned_only: boolean): void {
  filters.update((current) => ({ ...current, pinned_only }));
}

/** Enseña u oculta los proyectos ausentes. */
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

/** Cambia la dirección del orden. */
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

/** Pliega o despliega el panel lateral. */
export function toggleSidebar(): void {
  sidebarCollapsed.update((collapsed) => !collapsed);
}

/** Alterna entre las densidades cómoda y compacta. */
export function toggleDensity(): void {
  density.update((actual) => (actual === 'comodo' ? 'compacto' : 'comodo'));
}

/** Pasa al siguiente tema: oscuro, claro, sistema y vuelta a empezar. */
export function cycleTheme(): void {
  theme.update((actual) => THEMES[(THEMES.indexOf(actual) + 1) % THEMES.length] ?? 'dark');
}

/** Dirección natural de cada criterio: nombres de la A a la Z, fechas de la más reciente. */
function defaultDirectionFor(sort: SortOption): SortDirection {
  return sort === 'name' ? 'asc' : 'desc';
}

/** Lee la vista guardada; se espera antes de pintar para no reordenar el tablero después. */
export async function loadViewState(): Promise<void> {
  let raw: string | null;
  try {
    raw = await api.getViewState();
  } catch {
    // La vista es una comodidad: si no se puede leer, se arranca de cero en silencio.
    return;
  }
  if (raw === null) return;

  try {
    const parsed = sanitizeViewState(JSON.parse(raw));
    filters.set(parsed.filters);
    sidebarCollapsed.set(parsed.sidebar_collapsed);
    density.set(parsed.density);
    theme.set(parsed.theme);
    langPreference.set(parsed.lang);
  } catch {
    /* JSON ilegible: se queda lo que hay. */
  }
}

/** Guarda cada cambio de vista con debounce, después de `loadViewState`; devuelve cómo pararlo. */
export function startPersistingViewState(): () => void {
  const save = debounce(() => {
    void api.setViewState(JSON.stringify(currentViewState())).catch(() => {
      /* Perder la vista guardada no debe interrumpir al usuario. */
    });
  }, PERSIST_DELAY_MS);

  // Cada store emite al suscribirse, y eso no es un cambio. Se ignora lo que llega
  // mientras tanto en vez de contar emisiones: con un contador, quitar un store
  // dejó de guardar el primer cambio de verdad.
  let subscribing = true;
  const onChange = () => {
    if (!subscribing) save();
  };

  const unsubscribes = [
    filters.subscribe(onChange),
    sidebarCollapsed.subscribe(onChange),
    density.subscribe(onChange),
    theme.subscribe(onChange),
    langPreference.subscribe(onChange),
  ];
  subscribing = false;

  return () => {
    save.cancel();
    unsubscribes.forEach((unsubscribe) => unsubscribe());
  };
}

/** El `ViewState` que se guarda; exportado para que un test vigile que no falte ningún campo. */
export function currentViewState(): ViewState {
  return {
    filters: get(filters),
    sidebar_collapsed: get(sidebarCollapsed),
    density: get(density),
    theme: get(theme),
    lang: get(langPreference),
  };
}

/** Un `ViewState` válido a partir de lo guardado; lo que no cuadre toma su valor por defecto. */
export function sanitizeViewState(raw: unknown): ViewState {
  // Campo a campo: una vista de una versión anterior no puede romper el arranque.
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
    theme: oneOf(source.theme, THEMES, 'dark'),
    lang: oneOf(source.lang, LANG_PREFERENCES, 'system'),
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
