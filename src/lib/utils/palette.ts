/**
 * Qué enseña la paleta de comandos para un texto dado.
 *
 * Es una función pura: recibe las acciones y los proyectos, y una función de
 * búsqueda de proyectos (la del tablero), y devuelve la lista ordenada. Así
 * se prueba sin montar el componente.
 */

import type { ProjectWithTags } from '../types';
import { normalizeText } from './text';

/** Una acción de la paleta: algo que hace la aplicación. */
export interface PaletteAction {
  id: string;
  label: string;
  /** Otras palabras por las que se encuentra («refrescar» para «Releer Git»). */
  keywords?: string[];
  /** Atajo de teclado, si tiene, para enseñarlo al lado. */
  shortcut?: string;
}

/** Un resultado de la paleta. */
export type PaletteItem =
  | { kind: 'action'; action: PaletteAction }
  | { kind: 'project'; project: ProjectWithTags };

/** Cuántos proyectos recientes se enseñan sin texto. */
export const RECENT_LIMIT = 6;
/** Cuántos proyectos se enseñan como mucho al buscar. */
export const PROJECT_LIMIT = 8;

function matchesAction(action: PaletteAction, needle: string): boolean {
  return [action.label, ...(action.keywords ?? [])].some((text) =>
    normalizeText(text).includes(needle),
  );
}

/** Los abiertos más recientemente primero; los nunca abiertos, fuera. */
export function recentProjects(projects: ProjectWithTags[], limit: number): ProjectWithTags[] {
  return projects
    .filter((project) => project.last_opened_at !== null && !project.missing)
    .sort((a, b) => (b.last_opened_at ?? 0) - (a.last_opened_at ?? 0))
    .slice(0, limit);
}

/**
 * Resultados para un texto. Sin texto: todas las acciones y los proyectos
 * recientes. Con texto: las acciones que coinciden y, detrás, los proyectos
 * que encuentra `search` (la búsqueda difusa del tablero).
 */
export function paletteItems(
  query: string,
  actions: PaletteAction[],
  projects: ProjectWithTags[],
  search: (projects: ProjectWithTags[], query: string, limit: number) => ProjectWithTags[],
): PaletteItem[] {
  const needle = normalizeText(query.trim());
  const matchedActions = needle === '' ? actions : actions.filter((a) => matchesAction(a, needle));
  const matchedProjects =
    needle === '' ? recentProjects(projects, RECENT_LIMIT) : search(projects, query, PROJECT_LIMIT);

  return [
    ...matchedActions.map((action): PaletteItem => ({ kind: 'action', action })),
    ...matchedProjects.map((project): PaletteItem => ({ kind: 'project', project })),
  ];
}
