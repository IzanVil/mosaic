/**
 * Los proyectos, el escaneo y la lista visible.
 *
 * `projects` es la única fuente del tablero, con etiquetas y estado Git dentro.
 * Filtrar y ordenar solo se hace en `visibleProjects`: un componente que filtrase
 * por su cuenta discreparía del contador de la barra de búsqueda.
 */

import Fuse, { type IFuseOptions } from 'fuse.js';
import { derived, writable } from 'svelte/store';

import { listProjectsWithTags, setProjectPinned } from '../api/projects';
import { scanAllPaths } from '../api/scanner';
import type { FilterState, GitStatusEntry, ProjectWithTags, ScanSummary, Tag } from '../types';
import { errorText } from '../utils/errors';
import { compareNames, normalizeText } from '../utils/text';
import { filters } from './filters';

/** Todos los proyectos, con sus etiquetas y su estado Git. */
export const projects = writable<ProjectWithTags[]>([]);
/** La lista se está cargando. */
export const loadingProjects = writable(false);
/** Hay un escaneo en marcha. */
export const scanning = writable(false);
/** Resumen del último escaneo. */
export const lastScan = writable<ScanSummary | null>(null);
/** Último error de proyectos o de escaneo, para mostrarlo en la interfaz. */
export const projectsError = writable<string | null>(null);

/** Configuración de la búsqueda difusa. */
const FUSE_OPTIONS: IFuseOptions<IndexedProject> = {
  keys: ['name', 'path'],
  threshold: 0.3,
  ignoreLocation: true,
};

/** Lo que indexa la búsqueda difusa, con el texto ya normalizado. */
interface IndexedProject {
  id: number;
  name: string;
  path: string;
}

let indexCache: { source: ProjectWithTags[]; fuse: Fuse<IndexedProject> } | null = null;

/** El índice de búsqueda, que solo se rehace cuando cambia la lista. */
function searchIndex(source: ProjectWithTags[]): Fuse<IndexedProject> {
  // La clave es la identidad del array: reindexar en cada tecla se nota con cientos.
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

/** Proyectos que coinciden con un texto, del más parecido al menos; mismo índice que el tablero. */
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
      // Sin estado cacheado todavía no se sabe si está limpio: no cuenta como limpio.
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

/** Al ordenar por última apertura, los nunca abiertos van al final en las dos direcciones. */
function neverOpenedLast(a: ProjectWithTags, b: ProjectWithTags, current: FilterState): number {
  // No son «los más antiguos», son otra categoría: por eso esto va antes de la dirección.
  if (current.sort !== 'last_opened') return 0;
  const aNever = a.last_opened_at === null;
  const bNever = b.last_opened_at === null;
  if (aNever === bNever) return 0;
  return aNever ? 1 : -1;
}

/** La lista que pinta el tablero: filtrada, buscada y ordenada, con los fijados primero. */
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
    // El nombre desempata, para que dos proyectos con la misma fecha no bailen entre renders.
    return bySort !== 0 ? bySort : compareNames(a.name, b.name);
  });
});

/**
 * La misma lista en los dos grupos del tablero, fijados y resto, ordenados cada uno.
 * No se mezclan: así «nombre descendente» no tiene una excepción en la primera fila.
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
    projectsError.set(errorText(error));
  } finally {
    loadingProjects.set(false);
  }
}

/** Escanea y recarga la lista; `null` si falló. El refresco de Git lo encadena `App.svelte`. */
export async function runScan(): Promise<ScanSummary | null> {
  scanning.set(true);
  projectsError.set(null);
  try {
    const summary = await scanAllPaths();
    lastScan.set(summary);
    projects.set(await listProjectsWithTags());
    return summary;
  } catch (error) {
    projectsError.set(errorText(error));
    return null;
  } finally {
    scanning.set(false);
  }
}

/** Fija o deja de fijar un proyecto: primero en local, para que la tarjeta responda ya. */
export async function togglePinned(id: number, pinned: boolean): Promise<void> {
  patchProject(id, (project) => ({ ...project, pinned }));

  try {
    await setProjectPinned(id, pinned);
  } catch (error) {
    projectsError.set(errorText(error));
    await loadProjects();
  }
}

/** Refleja en el tablero las notas que acaba de guardar la vista de detalle. */
export function setProjectNotesLocally(id: number, notes: string | null): void {
  patchProject(id, (project) => ({ ...project, notes }));
}

/** Apunta en local la apertura que ya guardó el backend, para que la tarjeta y el orden la vean. */
export function markProjectOpened(id: number, openedAt: number): void {
  patchProject(id, (project) => ({ ...project, last_opened_at: openedAt }));
}
/**
 * Mezcla la caché de Git entera en los proyectos y dice cuántos han cambiado.
 * No reemplaza la lista: lo que no cambia conserva su objeto y su tarjeta no parpadea.
 */
export function mergeGitStatus(entries: GitStatusEntry[]): number {
  const byProject = new Map(entries.map((entry) => [entry.project_id, entry]));
  return replaceGitStatus((project) => byProject.get(project.id) ?? null);
}

/** Aplica el parche de un refresco, sin tocar los demás, y dice cuántos han cambiado. */
export function patchGitStatus(changed: GitStatusEntry[], removed: number[]): number {
  const byProject = new Map<number, GitStatusEntry | null>(
    changed.map((entry) => [entry.project_id, entry]),
  );
  for (const id of removed) byProject.set(id, null);
  return replaceGitStatus((project) =>
    byProject.has(project.id) ? (byProject.get(project.id) ?? null) : project.git_status,
  );
}

/** Pone a cada proyecto el estado que diga `next` y cuenta los que cambian. */
function replaceGitStatus(next: (project: ProjectWithTags) => GitStatusEntry | null): number {
  let changed = 0;
  projects.update((current) =>
    current.map((project) => {
      const status = next(project);
      if (sameGitStatus(project.git_status, status)) return project;
      changed += 1;
      return { ...project, git_status: status };
    }),
  );
  return changed;
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

/** Aplica un renombrado o un borrado de etiqueta a todos los proyectos que la llevan. */
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
  // Sin `refreshed_at`: es nueva en cada lectura, y el botón de Git no diría nunca «sin cambios».
  return (
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
