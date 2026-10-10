/**
 * Las reglas de qué tarjetas se ven en el tablero. `visibleProjects` es el
 * único sitio donde se filtra: si una regla se rompe, el tablero enseña de más
 * o de menos y el contador «X de Y» lo da por bueno.
 */

import { get } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../../src/lib/api/projects', () => ({
  listProjectsWithTags: vi.fn(async () => []),
  setProjectPinned: vi.fn(async () => undefined),
}));
vi.mock('../../src/lib/api/scanner', () => ({ scanAllPaths: vi.fn() }));

import { DEFAULT_FILTERS, filters } from '../../src/lib/stores/filters';
import { projects, visibleGroups, visibleProjects } from '../../src/lib/stores/projects';
import type { FilterState, ProjectWithTags, Tag } from '../../src/lib/types';

const cliente: Tag = { id: 1, name: 'cliente', color: '#3B82F6', created_at: 0 };
const personal: Tag = { id: 2, name: 'personal', color: '#8B5CF6', created_at: 0 };
const urgente: Tag = { id: 3, name: 'urgente', color: '#EF4444', created_at: 0 };

let nextId = 1;

function proyecto(name: string, extra: Partial<ProjectWithTags> = {}): ProjectWithTags {
  const id = nextId++;
  return {
    id,
    name,
    path: `/home/izan/proyectos/${name}`,
    is_git_repo: true,
    primary_language: 'Rust',
    last_opened_at: null,
    pinned: false,
    notes: null,
    missing: false,
    last_seen_at: 0,
    created_at: id,
    updated_at: id,
    tags: [],
    git_status: {
      project_id: id,
      refreshed_at: 0,
      branch: 'main',
      ahead: null,
      behind: null,
      is_dirty: false,
      last_commit_sha: null,
      last_commit_msg: null,
      last_commit_at: null,
      remote_url: null,
    },
    ...extra,
  };
}

function sucio(p: ProjectWithTags): ProjectWithTags {
  return { ...p, git_status: { ...p.git_status!, is_dirty: true } };
}

function visibles(cambios: Partial<FilterState> = {}): string[] {
  filters.set({ ...DEFAULT_FILTERS, ...cambios });
  return get(visibleProjects).map((p) => p.name);
}

beforeEach(() => {
  nextId = 1;
  filters.set({ ...DEFAULT_FILTERS });
});

describe('etiquetas y lenguajes', () => {
  beforeEach(() => {
    projects.set([
      proyecto('a-cliente', { tags: [cliente] }),
      proyecto('b-personal', { tags: [personal], primary_language: 'Go' }),
      proyecto('c-las-dos', { tags: [cliente, personal], primary_language: 'Python' }),
      proyecto('d-ninguna', { primary_language: null }),
    ]);
  });

  it('varias etiquetas se combinan en OR', () => {
    expect(visibles({ tag_ids: [cliente.id, personal.id] })).toEqual([
      'a-cliente',
      'b-personal',
      'c-las-dos',
    ]);
  });

  it('una etiqueta que nadie tiene no deja nada', () => {
    expect(visibles({ tag_ids: [urgente.id] })).toEqual([]);
  });

  it('varios lenguajes se combinan en OR', () => {
    expect(visibles({ languages: ['Go', 'Python'] })).toEqual(['b-personal', 'c-las-dos']);
  });

  it('un proyecto sin lenguaje no coincide con ningún filtro de lenguaje', () => {
    expect(visibles({ languages: ['Rust', 'Go', 'Python'] })).not.toContain('d-ninguna');
  });

  it('etiquetas y lenguajes se combinan entre sí en AND', () => {
    expect(visibles({ tag_ids: [cliente.id], languages: ['Python'] })).toEqual(['c-las-dos']);
  });
});

describe('estado de Git', () => {
  beforeEach(() => {
    projects.set([
      proyecto('limpio'),
      sucio(proyecto('sucio')),
      proyecto('sin-leer', { git_status: null }),
      proyecto('sin-repo', { is_git_repo: false, git_status: null }),
    ]);
  });

  it('«con cambios» solo deja los que tienen cambios', () => {
    expect(visibles({ git_state: 'dirty' })).toEqual(['sucio']);
  });

  it('«limpio» no cuenta un repositorio que todavía no se ha leído', () => {
    expect(visibles({ git_state: 'clean' })).toEqual(['limpio']);
  });

  it('«sin repositorio» deja solo las carpetas sin Git', () => {
    expect(visibles({ git_state: 'no_repo' })).toEqual(['sin-repo']);
  });

  it('«todos» no filtra', () => {
    expect(visibles({ git_state: 'all' })).toHaveLength(4);
  });

  it('se combina en AND con las etiquetas', () => {
    projects.update((lista) =>
      lista.map((p) => (p.name === 'sucio' ? p : { ...p, tags: [cliente] })),
    );
    expect(visibles({ git_state: 'dirty', tag_ids: [cliente.id] })).toEqual([]);
  });
});

describe('ausentes y fijados', () => {
  beforeEach(() => {
    projects.set([
      proyecto('en-disco'),
      proyecto('ausente', { missing: true }),
      proyecto('fijado', { pinned: true }),
    ]);
  });

  it('por defecto se ven también los ausentes', () => {
    expect(visibles()).toContain('ausente');
  });

  it('el filtro de ausentes los oculta', () => {
    expect(visibles({ include_missing: false })).not.toContain('ausente');
  });

  it('«solo fijados» deja solo los fijados', () => {
    expect(visibles({ pinned_only: true })).toEqual(['fijado']);
  });
});

describe('búsqueda', () => {
  beforeEach(() => {
    projects.set([
      proyecto('Ñandú-api'),
      proyecto('Cámara-Web'),
      proyecto('otro', { path: '/srv/clientes/facturación/otro' }),
    ]);
  });

  it('ignora acentos y mayúsculas en lo que se escribe', () => {
    expect(visibles({ query: 'CÁMARA' })).toEqual(['Cámara-Web']);
  });

  it('ignora acentos y mayúsculas en el nombre del proyecto', () => {
    expect(visibles({ query: 'nandu' })).toEqual(['Ñandú-api']);
  });

  it('también busca en la ruta', () => {
    expect(visibles({ query: 'facturacion' })).toEqual(['otro']);
  });

  it('una búsqueda de solo espacios no filtra', () => {
    expect(visibles({ query: '   ' })).toHaveLength(3);
  });

  it('se combina en AND con los filtros', () => {
    expect(visibles({ query: 'camara', git_state: 'dirty' })).toEqual([]);
  });
});

describe('orden y grupos', () => {
  beforeEach(() => {
    projects.set([
      proyecto('beta'),
      proyecto('zeta', { pinned: true }),
      proyecto('alfa'),
      proyecto('omega', { pinned: true }),
    ]);
  });

  it('los fijados van primero y el orden se aplica dentro de cada grupo', () => {
    expect(visibles({ sort: 'name', sort_dir: 'asc' })).toEqual(['omega', 'zeta', 'alfa', 'beta']);
  });

  it('al invertir el orden los fijados siguen arriba', () => {
    expect(visibles({ sort: 'name', sort_dir: 'desc' })).toEqual([
      'zeta',
      'omega',
      'beta',
      'alfa',
    ]);
  });

  it('los dos grupos del tablero no se mezclan', () => {
    filters.set({ ...DEFAULT_FILTERS, sort_dir: 'desc' });
    const grupos = get(visibleGroups);
    expect(grupos.pinned.map((p) => p.name)).toEqual(['zeta', 'omega']);
    expect(grupos.rest.map((p) => p.name)).toEqual(['beta', 'alfa']);
  });

  it('por última apertura, los nunca abiertos van al final en las dos direcciones', () => {
    projects.set([
      proyecto('nunca'),
      proyecto('ayer', { last_opened_at: 100 }),
      proyecto('hoy', { last_opened_at: 200 }),
    ]);
    expect(visibles({ sort: 'last_opened', sort_dir: 'asc' })).toEqual(['ayer', 'hoy', 'nunca']);
    expect(visibles({ sort: 'last_opened', sort_dir: 'desc' })).toEqual(['hoy', 'ayer', 'nunca']);
  });

  it('con la misma fecha desempata el nombre, para que el orden no baile', () => {
    projects.set([
      proyecto('b', { created_at: 5 }),
      proyecto('a', { created_at: 5 }),
      proyecto('c', { created_at: 1 }),
    ]);
    expect(visibles({ sort: 'created', sort_dir: 'asc' })).toEqual(['c', 'a', 'b']);
  });
});
