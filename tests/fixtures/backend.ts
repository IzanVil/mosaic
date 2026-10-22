/**
 * Backend simulado para los tests de extremo a extremo.
 *
 * `installBackend` se inyecta en la página antes que la app
 * (`page.addInitScript`) y hace de `window.__TAURI_INTERNALS__`, que es por
 * donde pasa todo `invoke()`. Responde con un estado en memoria que los
 * propios comandos modifican, y lo guarda en `sessionStorage`: recargar la
 * página es como cerrar y abrir la app, la «base de datos» sigue ahí.
 *
 * Solo simula los comandos que usan los recorridos. Uno que no esté rechaza la
 * promesa con «no simulado», y el test lo enseña en vez de pasar por error.
 *
 * La función se serializa entera al inyectarla: no puede usar nada de fuera
 * de su cuerpo salvo tipos.
 */

import type { ProjectWithTags, ScanPath, ScanSummary, Tag } from '../../src/lib/types';

/** Un proyecto que el escáner encontraría en la carpeta elegida. */
export interface DiskProject {
  name: string;
  language: string | null;
  dirty: boolean;
}

/** Cómo arranca el backend. */
export interface Seed {
  /** Carpeta que devuelve el diálogo de «Añadir carpeta». */
  folder: string;
  /** Lo que hay dentro de `folder`. */
  disk: DiskProject[];
  /** Arrancar con `folder` ya añadida y escaneada, o con la app recién instalada. */
  scanned: boolean;
}

interface State {
  scanPaths: ScanPath[];
  projects: ProjectWithTags[];
  tags: Tag[];
  viewState: string | null;
  nextId: number;
}

type Args = Record<string, unknown>;

export function installBackend(seed: Seed): void {
  const KEY = 'mosaic-e2e';
  const NOW = 1_760_000_000;

  const saved = sessionStorage.getItem(KEY);
  const state: State = saved
    ? JSON.parse(saved)
    : { scanPaths: [], projects: [], tags: [], viewState: null, nextId: 1 };

  const nextId = () => state.nextId++;
  const project = (id: unknown) => {
    const found = state.projects.find((p) => p.id === id);
    if (!found) throw { code: 'project_not_found', params: { id }, message: `no existe el proyecto ${id}` };
    return found;
  };

  function addScanPath(path: string): ScanPath {
    const added = { id: nextId(), path, enabled: true, created_at: NOW, last_scan_at: null };
    state.scanPaths.push(added);
    return added;
  }

  function scan(): ScanSummary {
    let found = 0;
    let isNew = 0;
    for (const root of state.scanPaths.filter((p) => p.enabled)) {
      if (root.path !== seed.folder) continue;
      root.last_scan_at = NOW;
      for (const disk of seed.disk) {
        found += 1;
        const path = `${root.path}/${disk.name}`;
        if (state.projects.some((p) => p.path === path)) continue;
        isNew += 1;
        const id = nextId();
        state.projects.push({
          id,
          name: disk.name,
          path,
          is_git_repo: true,
          primary_language: disk.language,
          last_opened_at: null,
          pinned: false,
          notes: null,
          missing: false,
          last_seen_at: NOW,
          created_at: NOW,
          updated_at: NOW,
          tags: [],
          git_status: {
            project_id: id,
            refreshed_at: NOW,
            branch: 'main',
            ahead: null,
            behind: null,
            is_dirty: disk.dirty,
            last_commit_sha: 'abc1234',
            last_commit_msg: 'chore: inicial',
            last_commit_at: NOW - 3600,
            remote_url: null,
          },
        });
      }
    }
    return {
      roots_scanned: state.scanPaths.length,
      roots_unavailable: 0,
      entries_visited: found * 10,
      projects_found: found,
      projects_new: isNew,
      projects_updated: 0,
      projects_missing: 0,
      truncated: false,
      elapsed_ms: 12,
    };
  }

  if (!saved && seed.scanned) {
    addScanPath(seed.folder);
    scan();
  }

  const advanced = {
    max_depth: 4,
    excluded_dirs: ['.git', 'node_modules', 'target'],
    max_entries_per_scan: 50_000,
    scan_on_startup: false,
    git_refresh_interval_minutes: 5,
  };

  const commands: Record<string, (args: Args) => unknown> = {
    get_view_state: () => state.viewState,
    set_view_state: ({ json }) => {
      state.viewState = json as string;
      return null;
    },
    list_projects_with_tags: () => state.projects,
    set_project_pinned: ({ id, pinned }) => {
      project(id).pinned = pinned as boolean;
      return null;
    },
    set_project_notes: ({ id, notes }) => {
      const text = notes as string;
      project(id).notes = text.trim() === '' ? null : text;
      return project(id).notes;
    },
    get_project_readme: () => null,
    get_project_history: () => [],
    get_project_branches: () => ({ local: [], remote: [] }),
    list_scan_paths: () => state.scanPaths,
    add_scan_path: ({ path }) => addScanPath(path as string),
    scan_all_paths: () => scan(),
    list_git_status: () => state.projects.flatMap((p) => (p.git_status ? [p.git_status] : [])),
    refresh_all_git_status: () => ({
      refreshed: state.projects.length,
      failed: 0,
      elapsed_ms: 3,
      changed: [],
      removed: [],
    }),
    list_tags: () =>
      state.tags.map((tag) => ({
        ...tag,
        project_count: state.projects.filter((p) => p.tags.some((t) => t.id === tag.id)).length,
      })),
    create_tag: ({ name, color }) => {
      const wanted = (name as string).trim();
      if (state.tags.some((t) => t.name.toLowerCase() === wanted.toLowerCase())) {
        throw { code: 'tag_name_taken', params: { name: wanted }, message: `ya existe una etiqueta llamada «${wanted}»` };
      }
      const tag = { id: nextId(), name: wanted, color: color as string, created_at: NOW };
      state.tags.push(tag);
      return tag;
    },
    assign_tag: ({ projectId, tagId }) => {
      const target = project(projectId);
      const tag = state.tags.find((t) => t.id === tagId);
      if (tag && !target.tags.some((t) => t.id === tag.id)) {
        target.tags = [...target.tags, tag].sort((a, b) => a.name.localeCompare(b.name));
      }
      return null;
    },
    unassign_tag: ({ projectId, tagId }) => {
      const target = project(projectId);
      target.tags = target.tags.filter((t) => t.id !== tagId);
      return null;
    },
    list_detected_apps: () => [],
    get_preferred_apps: () => ({ ide: '', terminal: '' }),
    get_advanced_settings: () => ({
      current: advanced,
      defaults: advanced,
      limits: {
        max_depth_min: 1,
        max_depth_max: 8,
        max_entries_min: 1000,
        max_entries_max: 500_000,
        git_refresh_max_minutes: 60,
        excluded_dirs_max: 100,
        excluded_dir_max_chars: 255,
      },
    }),
    'plugin:dialog|open': () => seed.folder,
  };

  let callbacks = 0;
  Object.assign(window, {
    __TAURI_INTERNALS__: {
      transformCallback: () => ++callbacks,
      invoke(command: string, args: Args = {}) {
        const handler = commands[command];
        if (handler) {
          try {
            const result = handler(args);
            sessionStorage.setItem(KEY, JSON.stringify(state));
            // Una copia, como la que llega por IPC: si la app recibiera los
            // objetos del estado, cada comando se los cambiaría por debajo.
            return Promise.resolve(result === undefined ? null : structuredClone(result));
          } catch (error) {
            return Promise.reject(error);
          }
        }
        // Escuchar eventos y el portapapeles: basta con que respondan.
        if (command.startsWith('plugin:')) return Promise.resolve(1);
        return Promise.reject(`no simulado: ${command}`);
      },
      metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main' } },
    },
    __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener: () => {} },
  });
}
