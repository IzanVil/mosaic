/**
 * El refresco de Git llega como un parche: solo lo que cambió y lo que se
 * retiró de la caché. Si se aplica mal, el tablero enseña un estado viejo, o
 * el botón de Git dice que actualizó repositorios que no cambiaron.
 */

import { get } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../../src/lib/api/git', () => ({
  listGitStatus: vi.fn(),
  refreshAllGitStatus: vi.fn(),
}));
vi.mock('../../src/lib/api/projects', () => ({ listProjectsWithTags: vi.fn(async () => []) }));
vi.mock('../../src/lib/api/scanner', () => ({ scanAllPaths: vi.fn() }));

import * as api from '../../src/lib/api/git';
import { applyGitRefresh, lastGitRefresh, refreshAllGitStatus } from '../../src/lib/stores/gitStatus';
import { mergeGitStatus, projects } from '../../src/lib/stores/projects';
import type { GitRefreshSummary, GitStatusEntry, ProjectWithTags } from '../../src/lib/types';

function estado(id: number, extra: Partial<GitStatusEntry> = {}): GitStatusEntry {
  return {
    project_id: id,
    refreshed_at: 100,
    branch: 'main',
    ahead: null,
    behind: null,
    is_dirty: false,
    last_commit_sha: 'abc1234',
    last_commit_msg: 'chore: inicial',
    last_commit_at: 50,
    remote_url: null,
    ...extra,
  };
}

function proyecto(id: number): ProjectWithTags {
  return {
    id,
    name: `p${id}`,
    path: `/home/izan/proyectos/p${id}`,
    is_git_repo: true,
    primary_language: 'Rust',
    last_opened_at: null,
    pinned: false,
    notes: null,
    missing: false,
    last_seen_at: 0,
    created_at: 0,
    updated_at: 0,
    tags: [],
    git_status: estado(id),
  };
}

function resumen(extra: Partial<GitRefreshSummary> = {}): GitRefreshSummary {
  return { refreshed: 2, failed: 0, elapsed_ms: 1, changed: [], removed: [], ...extra };
}

beforeEach(() => {
  vi.mocked(api.listGitStatus).mockReset();
  vi.mocked(api.refreshAllGitStatus).mockReset();
  projects.set([proyecto(1), proyecto(2)]);
  lastGitRefresh.set(null);
});

describe('el botón de Git', () => {
  it('dice «sin cambios» si el backend no trae ninguno, y no reescribe ningún proyecto', async () => {
    const antes = get(projects);
    vi.mocked(api.refreshAllGitStatus).mockResolvedValue(resumen());

    await refreshAllGitStatus();

    expect(get(lastGitRefresh)).toMatchObject({ read: 2, changed: 0, manual: true });
    expect(get(projects)[0]).toBe(antes[0]);
    expect(get(projects)[1]).toBe(antes[1]);
  });

  it('ya no pide la caché entera después de releer', async () => {
    vi.mocked(api.refreshAllGitStatus).mockResolvedValue(resumen());

    await refreshAllGitStatus();

    expect(api.listGitStatus).not.toHaveBeenCalled();
  });
});

describe('el parche de un refresco', () => {
  it('solo reescribe los proyectos que cambiaron', () => {
    const antes = get(projects);

    applyGitRefresh(resumen({ changed: [estado(2, { is_dirty: true, refreshed_at: 200 })] }));

    const despues = get(projects);
    expect(despues[0]).toBe(antes[0]);
    expect(despues[1].git_status?.is_dirty).toBe(true);
    expect(get(lastGitRefresh)).toMatchObject({ changed: 1, manual: false });
  });

  it('deja sin estado a un proyecto que ya no se puede leer', () => {
    applyGitRefresh(resumen({ refreshed: 1, failed: 1, removed: [1] }));

    expect(get(projects)[0].git_status).toBeNull();
    expect(get(projects)[1].git_status).not.toBeNull();
    expect(get(lastGitRefresh)).toMatchObject({ read: 1, failed: 1, changed: 1 });
  });

  it('ignora los proyectos que todavía no están en el tablero', () => {
    const antes = get(projects);

    applyGitRefresh(resumen({ changed: [estado(99, { is_dirty: true })], removed: [98] }));

    expect(get(projects)).toEqual(antes);
    expect(get(lastGitRefresh)?.changed).toBe(0);
  });
});

describe('la cuenta de cambios', () => {
  it('no cuenta una entrada que solo trae una hora de lectura nueva', () => {
    const antes = get(projects);

    const cambiados = mergeGitStatus([estado(1, { refreshed_at: 999 }), estado(2, { refreshed_at: 999 })]);

    expect(cambiados).toBe(0);
    expect(get(projects)[0]).toBe(antes[0]);
  });
});
