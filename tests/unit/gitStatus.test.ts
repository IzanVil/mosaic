/**
 * El estado Git se mezcla en el tablero. Si la mezcla cuenta mal, el botón de
 * Git dice que actualizó repositorios que no cambiaron.
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
import { lastGitRefresh, refreshAllGitStatus } from '../../src/lib/stores/gitStatus';
import { mergeGitStatus, projects } from '../../src/lib/stores/projects';
import type { GitStatusEntry, ProjectWithTags } from '../../src/lib/types';

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

beforeEach(() => {
  vi.mocked(api.listGitStatus).mockReset();
  vi.mocked(api.refreshAllGitStatus).mockReset();
  projects.set([proyecto(1), proyecto(2)]);
  lastGitRefresh.set(null);
});

describe('el botón de Git', () => {
  it('dice «sin cambios» si al releer solo cambia la hora de lectura', async () => {
    const antes = get(projects);
    vi.mocked(api.refreshAllGitStatus).mockResolvedValue({ refreshed: 2, failed: 0, elapsed_ms: 1 });
    vi.mocked(api.listGitStatus).mockResolvedValue([
      estado(1, { refreshed_at: 200 }),
      estado(2, { refreshed_at: 200 }),
    ]);

    await refreshAllGitStatus();

    expect(get(lastGitRefresh)).toMatchObject({ read: 2, changed: 0, manual: true });
    expect(get(projects)[0]).toBe(antes[0]);
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
