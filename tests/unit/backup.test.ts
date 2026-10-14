import { get } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../../src/lib/api/backup', () => ({
  exportBackup: vi.fn(async () => ({ scan_paths: 2, tags: 3, projects: 1 })),
  previewImport: vi.fn(),
  applyImport: vi.fn(),
}));
vi.mock('../../src/lib/stores/advancedSettings', () => ({ loadAdvancedSettings: vi.fn() }));
vi.mock('../../src/lib/stores/apps', () => ({ loadApps: vi.fn() }));
vi.mock('../../src/lib/stores/projects', () => ({ loadProjects: vi.fn() }));
vi.mock('../../src/lib/stores/scanPaths', () => ({ loadScanPaths: vi.fn() }));
vi.mock('../../src/lib/stores/tags', () => ({ loadTags: vi.fn() }));

import { applyImport, previewImport } from '../../src/lib/api/backup';
import { loadProjects } from '../../src/lib/stores/projects';
import { loadTags } from '../../src/lib/stores/tags';
import { backupState, confirmImport, exportTo, previewFrom } from '../../src/lib/stores/backup';
import { backupFileName, describeExport, describePlan } from '../../src/lib/utils/backup';
import type { ImportPlan } from '../../src/lib/types';

function plan(overrides: Partial<ImportPlan> = {}): ImportPlan {
  return {
    exported_at: 0,
    app_version: '0.1.0',
    has_work: true,
    settings_changed: [],
    git_interval_changed: false,
    scan_paths_added: 0,
    scan_paths_missing: [],
    tags_added: 0,
    tags_existing: 0,
    projects_matched: 0,
    projects_not_found: 0,
    pins_added: 0,
    assignments_added: 0,
    notes_added: 0,
    notes_conflicts: [],
    ...overrides,
  };
}

beforeEach(() => {
  vi.clearAllMocks();
  backupState.set({ step: 'idle' });
});

describe('describePlan', () => {
  it('dice que no cambia nada si no hay nada que importar', () => {
    expect(describePlan(plan(), 'es')).toEqual([
      { tone: 'note', text: 'No cambia nada: ya tienes todo lo que trae esta copia.' },
    ]);
  });

  it('pone los cambios antes que los avisos, con singular y plural', () => {
    const lines = describePlan(
      plan({
        settings_changed: ['max_depth', 'preferred_ide'],
        tags_added: 1,
        assignments_added: 3,
        notes_conflicts: ['api'],
        scan_paths_missing: ['/a', '/b'],
      }),
      'es',
    );
    expect(lines.map((l) => l.text)).toEqual([
      'Ajustes: cambia profundidad, editor.',
      'Crea 1 etiqueta.',
      'Pone 3 etiquetas a tus proyectos.',
      'api: ya tiene notas distintas. Se quedan las tuyas.',
      '2 rutas no existen en este equipo y se saltan: /a, /b.',
    ]);
    expect(lines.map((l) => l.tone)).toEqual(['change', 'change', 'change', 'warning', 'warning']);
  });

  it('avisa de que la copia no trae trabajo', () => {
    const lines = describePlan(plan({ has_work: false, scan_paths_added: 1 }), 'es');
    expect(lines.at(-1)?.text).toBe('La copia no trae etiquetas ni notas.');
  });

  it('en inglés dice lo mismo, con sus plurales', () => {
    const lines = describePlan(
      plan({
        settings_changed: ['max_depth', 'preferred_ide'],
        tags_added: 1,
        notes_conflicts: ['api', 'web'],
        scan_paths_missing: ['/a'],
        tags_existing: 2,
      }),
      'en',
    );
    expect(lines.map((l) => l.text)).toEqual([
      'Settings: changes depth, editor.',
      'Creates 1 tag.',
      'api, web: already have different notes. Yours are kept.',
      "1 folder doesn't exist on this computer and is skipped: /a.",
      '2 tags already existed and keep their color.',
    ]);
  });

  it('un ajuste que no conoce se enseña por su nombre', () => {
    expect(describePlan(plan({ settings_changed: ['nuevo_ajuste'] }), 'es')[0]?.text).toBe(
      'Ajustes: cambia nuevo_ajuste.',
    );
  });
});

describe('textos de exportar', () => {
  it('propone un nombre con la fecha', () => {
    expect(backupFileName(new Date(2026, 9, 7), 'es')).toBe('mosaic-copia-2026-10-07.json');
    expect(backupFileName(new Date(2026, 9, 7), 'en')).toBe('mosaic-backup-2026-10-07.json');
  });

  it('no menciona etiquetas si no se incluyeron', () => {
    expect(describeExport({ scan_paths: 1, tags: 0, projects: 0 }, false, 'es')).toBe(
      'Copia guardada: ajustes y 1 ruta.',
    );
    expect(describeExport({ scan_paths: 1, tags: 0, projects: 0 }, false, 'en')).toBe(
      'Backup saved: settings and 1 folder.',
    );
  });
});

describe('store de la copia', () => {
  it('exportar deja el resumen a la vista, sin escribirlo todavía en un idioma', async () => {
    await exportTo('/tmp/c.json', true);
    const summary = { scan_paths: 2, tags: 3, projects: 1 };
    expect(get(backupState)).toEqual({ step: 'exported', summary, includeWork: true });
    expect(describeExport(summary, true, 'es')).toBe(
      'Copia guardada: ajustes, 2 rutas, 3 etiquetas y 1 proyecto con etiquetas, notas o fijados.',
    );
    expect(describeExport(summary, true, 'en')).toBe(
      'Backup saved: settings, 2 folders, 3 tags and 1 project with tags, notes or pins.',
    );
  });

  it('previsualizar no importa', async () => {
    vi.mocked(previewImport).mockResolvedValue(plan({ tags_added: 2 }));
    await previewFrom('/tmp/c.json');
    expect(get(backupState)).toMatchObject({ step: 'preview', path: '/tmp/c.json' });
    expect(applyImport).not.toHaveBeenCalled();
  });

  it('importar recarga lo que puede haber cambiado', async () => {
    vi.mocked(applyImport).mockResolvedValue(plan({ tags_added: 2 }));
    await confirmImport('/tmp/c.json');
    expect(loadTags).toHaveBeenCalled();
    expect(loadProjects).toHaveBeenCalled();
    expect(get(backupState)).toMatchObject({ step: 'imported' });
  });

  it('un fichero ajeno se queda en error sin recargar nada', async () => {
    vi.mocked(previewImport).mockRejectedValue('el fichero no es una copia de Mosaic');
    await previewFrom('/tmp/foto.png');
    expect(get(backupState)).toEqual({
      step: 'error',
      message: 'el fichero no es una copia de Mosaic',
    });
    expect(loadTags).not.toHaveBeenCalled();
  });
});
