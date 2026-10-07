/**
 * Lo que enseña la paleta: acciones primero, después proyectos; sin texto,
 * los recientes; con texto, la misma búsqueda difusa que el tablero.
 */

import { describe, expect, it } from 'vitest';

import type { ProjectWithTags } from '../../src/lib/types';
import { paletteItems, recentProjects, type PaletteAction } from '../../src/lib/utils/palette';

const project = (id: number, name: string, opened: number | null, missing = false): ProjectWithTags => ({
  id,
  name,
  path: `/p/${name}`,
  is_git_repo: true,
  primary_language: null,
  last_opened_at: opened,
  pinned: false,
  notes: null,
  missing,
  last_seen_at: 0,
  created_at: 0,
  updated_at: 0,
  tags: [],
  git_status: null,
});

const actions: PaletteAction[] = [
  { id: 'git', label: 'Releer Git', keywords: ['refrescar'] },
  { id: 'scan', label: 'Escanear ahora' },
  { id: 'settings', label: 'Ir a Ajustes', keywords: ['preferencias'] },
];

const proyectos = [
  project(1, 'mosaic', 300),
  project(2, 'api-facturas', 100),
  project(3, 'nunca-abierto', null),
  project(4, 'borrado', 500, true),
];

const buscar = (source: ProjectWithTags[], query: string) =>
  source.filter((p) => p.name.includes(query.trim()));

describe('paletteItems', () => {
  it('sin texto enseña todas las acciones y los recientes, sin los nunca abiertos ni los ausentes', () => {
    const items = paletteItems('', actions, proyectos, buscar);
    expect(items.filter((i) => i.kind === 'action')).toHaveLength(3);
    expect(items.filter((i) => i.kind === 'project').map((i) => i.kind === 'project' && i.project.name)).toEqual([
      'mosaic',
      'api-facturas',
    ]);
  });

  it('con texto filtra acciones por nombre y por palabras clave, sin acentos', () => {
    const porClave = paletteItems('REFRESCAR', actions, proyectos, buscar);
    expect(porClave.map((i) => i.kind === 'action' && i.action.id)).toEqual(['git']);
    const porNombre = paletteItems('ajustes', actions, proyectos, buscar);
    expect(porNombre[0]).toEqual({ kind: 'action', action: actions[2] });
  });

  it('pone los proyectos detrás de las acciones', () => {
    const items = paletteItems('a', actions, proyectos, buscar);
    const primerProyecto = items.findIndex((i) => i.kind === 'project');
    const ultimaAccion = items.map((i) => i.kind).lastIndexOf('action');
    expect(primerProyecto).toBeGreaterThan(ultimaAccion);
  });
});

describe('recentProjects', () => {
  it('ordena del más reciente al más antiguo y respeta el límite', () => {
    expect(recentProjects(proyectos, 1).map((p) => p.name)).toEqual(['mosaic']);
  });
});
