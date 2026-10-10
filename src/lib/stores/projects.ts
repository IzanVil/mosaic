/**
 * Estado global de los proyectos, del escaneo y de la lista visible.
 *
 * `projects` es la única fuente de verdad del tablero: cada elemento llega del
 * backend con sus etiquetas y su estado Git ya dentro (`list_projects_with_tags`).
 *
 * Todo el filtrado y la ordenación viven en el store derivado
 * [`visibleProjects`], y en ningún otro sitio: un componente que filtrase por su
 * cuenta acabaría discrepando del contador de resultados de la barra de
 * búsqueda.
 */

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
/** Último error de proyectos o de escaneo, para mostrarlo en la interfaz. */
export const projectsError = writable<string | null>(null);

/** Configuración de la búsqueda difusa. */
const FUSE_OPTIONS: IFuseOptions<IndexedProject> = {
  keys: ['name', 'path'],
  threshold: 0.3,
  ignoreLocation: true,
};

/**
 * Índice de búsqueda, reconstruido solo cuando cambia la lista de proyectos.
 *
 * Reindexar en cada pulsación de tecla con cientos de proyectos se nota; la
 * clave es la identidad del array, que solo cambia cuando el store se reescribe.
 */
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

/**
 * Proyectos que coinciden con un texto, del más parecido al menos, hasta
 * `limit`. Usa el mismo índice que la búsqueda del tablero, así que la paleta
 * de comandos y la barra de búsqueda nunca discrepan.
 */
export function rankProjects(
  source: ProjectWithTags[],
  query: string,
  limit: number,
): ProjectWithTags[] {
  const needle = normalizeText(query.trim());
  if (needle === '') return [];
  const byId = new Map(source.map((project) => [project.id, project]));
  return searchIndex(source)
    .search(needle, { limit })
    .map((result) => byId.get(result.item.id))
    .filter((project): project is ProjectWithTags => project !== undefined);
}

/** Ids que coinciden con la búsqueda, o `null` si no hay búsqueda activa. */
function matchingIds(source: ProjectWithTags[], query: string): Set<number> | null {
  const needle = normalizeText(query.trim());
  if (needle === '') return null;

  const results = searchIndex(source).search(needle);
  return new Set(results.map((result) => result.item.id));
}

function matchesFilters(project: ProjectWithTags, current: FilterState): boolean {
  if (!current.include_missing && project.missing) return false;
  if (current.pinned_only && !project.pinned) return false;

  // Varias etiquetas se combinan en OR: vale con tener cualquiera de ellas.
  if (current.tag_ids.length > 0) {
    if (!project.tags.some((tag) => current.tag_ids.includes(tag.id))) return false;
  }

  // Los lenguajes también en OR; un proyecto sin lenguaje nunca coincide.
  if (current.languages.length > 0) {
    if (project.primary_language === null) return false;
    if (!current.languages.includes(project.primary_language)) return false;
  }

  switch (current.git_state) {
    case 'dirty':
      return project.git_status?.is_dirty === true;
    case 'clean':
      // Un repositorio sin estado cacheado todavía no se sabe si está limpio,
      // así que no cuenta como limpio.
      return project.is_git_repo && project.git_status?.is_dirty === false;
    case 'no_repo':
      return !project.is_git_repo;
    case 'all':
      return true;
  }
}

/** Compara dos proyectos por el criterio activo, sin mirar la dirección. */
function compareBy(a: ProjectWithTags, b: ProjectWithTags, current: FilterState): number {
  switch (current.sort) {
    case 'name':
      return compareNames(a.name, b.name);
    case 'last_opened':
      return (a.last_opened_at ?? 0) - (b.last_opened_at ?? 0);
    case 'created':
      return a.created_at - b.created_at;
    case 'updated':
      return a.updated_at - b.updated_at;
  }
}

/**
 * Manda al final los proyectos que nunca se han abierto, al ordenar por última
 * apertura. Un proyecto sin abrir no es «el más antiguo»: es otra categoría, y
 * queda al final en las dos direcciones, así que esto se aplica antes de la
 * dirección y no se invierte con ella.
 */
function neverOpenedLast(a: ProjectWithTags, b: ProjectWithTags, current: FilterState): number {
  if (current.sort !== 'last_opened') return 0;
  const aNever = a.last_opened_at === null;
  const bNever = b.last_opened_at === null;
  if (aNever === bNever) return 0;
  return aNever ? 1 : -1;
}

/**
 * La lista que pinta el tablero: filtrada, buscada y ordenada.
 *
 * Los proyectos fijados van siempre primero, como en la Fase 3: son los que el
 * usuario quiere tener a mano, y la tarjeta lo explica con su borde de acento.
 */
export const visibleProjects = derived([projects, filters], ([source, current]) => {
  const matches = matchingIds(source, current.query);

  const filtered = source.filter(
    (project) =>
      (matches === null || matches.has(project.id)) && matchesFilters(project, current),
  );

  const direction = current.sort_dir === 'asc' ? 1 : -1;
  return filtered.sort((a, b) => {
    if (a.pinned !== b.pinned) return a.pinned ? -1 : 1;

    const never = neverOpenedLast(a, b, current);
    if (never !== 0) return never;

    const bySort = compareBy(a, b, current) * direction;
    // El nombre desempata para que el orden no baile entre renders cuando dos
    // proyectos comparten fecha.
    return bySort !== 0 ? bySort : compareNames(a.name, b.name);
  });
});

/**
 * La misma lista, partida en los dos grupos que pinta el tablero.
 *
 * Los fijados van siempre arriba, en su propio bloque y con la ordenación
 * elegida aplicada dentro de cada grupo. Nunca se mezclan: así «ordenar por
 * nombre descendente» no tiene una excepción inexplicable en la primera fila.
 */
export const visibleGroups = derived(visibleProjects, (list) => ({
  pinned: list.filter((project) => project.pinned),
  rest: list.filter((project) => !project.pinned),
}));

/** Lenguajes presentes en los proyectos, para el desplegable de filtros. */
export const availableLanguages = derived(projects, (source) => {
  const languages = new Set<string>();
  for (const project of source) {
    if (project.primary_language !== null) languages.add(project.primary_language);
  }
  return [...languages].sort(compareNames);
});

/** Proyectos fijados, para la sección del sidebar. */
export const pinnedProjects = derived(projects, (source) =>
  source.filter((project) => project.pinned),
);

/** Recarga la lista de proyectos, con etiquetas y estado Git, desde la base de datos. */
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

/**
 * Lanza un escaneo completo y refresca la lista al terminar.
 *
 * Devuelve el resumen, o `null` si el escaneo falló. El refresco del estado Git
 * de los proyectos nuevos lo encadena quien llama (`App.svelte`), que es quien
 * puede indicar que el botón de Git está trabajando.
 */
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

/**
 * Fija o deja de fijar un proyecto.
 *
 * Actualiza la lista en local antes de esperar al backend para que la tarjeta
 * responda al instante, y recarga si la escritura falla.
 */
export async function togglePinned(id: number, pinned: boolean): Promise<void> {
  patchProject(id, (project) => ({ ...project, pinned }));

  try {
    await setProjectPinned(id, pinned);
  } catch (error) {
    projectsError.set(String(error));
    await loadProjects();
  }
}

/** Refleja en el tablero las notas que acaba de guardar la vista de detalle. */
export function setProjectNotesLocally(id: number, notes: string | null): void {
  patchProject(id, (project) => ({ ...project, notes }));
}

/**
 * Registra en local que un proyecto se acaba de abrir.
 *
 * El backend ya ha movido `last_opened_at`; sin esto la tarjeta seguiría
 * diciendo «Sin abrir desde Mosaic» y la ordenación por última apertura no se
 * movería hasta el siguiente arranque.
 */
export function markProjectOpened(id: number, openedAt: number): void {
  patchProject(id, (project) => ({ ...project, last_opened_at: openedAt }));
}
/**
 * Mezcla la caché de Git recién leída en los proyectos que ya están en memoria
 * y devuelve cuántos han cambiado de verdad.
 *
 * **No reemplaza la lista.** El refresco automático entra cada cinco minutos y
 * reescribir el array completo haría parpadear las tarjetas de cientos de
 * proyectos; además, los objetos que no cambian conservan su identidad, así que
 * Svelte no vuelve a montar sus tarjetas. Ese mismo cálculo alimenta el «sin
 * cambios» que muestra el botón de Git.
 */
export function mergeGitStatus(entries: GitStatusEntry[]): number {
  const byProject = new Map(entries.map((entry) => [entry.project_id, entry]));
  let changed = 0;
  projects.update((current) =>
    current.map((project) => {
      const next = byProject.get(project.id) ?? null;
      if (sameGitStatus(project.git_status, next)) return project;
      changed += 1;
      return { ...project, git_status: next };
    }),
  );
  return changed;
}

/** Mezcla el estado Git de un único proyecto. */
export function mergeOneGitStatus(projectId: number, entry: GitStatusEntry | null): void {
  patchProject(projectId, (project) =>
    sameGitStatus(project.git_status, entry) ? project : { ...project, git_status: entry },
  );
}

/** Añade una etiqueta a un proyecto en memoria, manteniendo el orden por nombre. */
export function attachTag(projectId: number, tag: Tag): void {
  patchProject(projectId, (project) =>
    project.tags.some((existing) => existing.id === tag.id)
      ? project
      : { ...project, tags: [...project.tags, tag].sort((a, b) => compareNames(a.name, b.name)) },
  );
}

/** Quita una etiqueta de un proyecto en memoria. */
export function detachTag(projectId: number, tagId: number): void {
  patchProject(projectId, (project) => ({
    ...project,
    tags: project.tags.filter((tag) => tag.id !== tagId),
  }));
}

/**
 * Aplica un cambio de etiqueta a todos los proyectos que la llevan.
 *
 * Lo usan el renombrado y el borrado desde el gestor: sin esto, las tarjetas
 * seguirían mostrando el nombre o el color viejos hasta la siguiente recarga.
 */
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

/** Reemplaza un proyecto por el resultado de `change`, si está en la lista. */
function patchProject(
  id: number,
  change: (project: ProjectWithTags) => ProjectWithTags,
): void {
  projects.update((current) =>
    current.map((project) => (project.id === id ? change(project) : project)),
  );
}

/** Compara dos entradas de caché para no reescribir un proyecto sin motivo. */
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
