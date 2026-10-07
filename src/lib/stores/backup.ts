/** Exportar e importar la copia de seguridad desde Ajustes. */

import { writable } from 'svelte/store';

import { applyImport, exportBackup, previewImport } from '../api/backup';
import type { ImportPlan } from '../types';
import { describeExport } from '../utils/backup';
import { loadAdvancedSettings } from './advancedSettings';
import { loadApps } from './apps';
import { loadProjects } from './projects';
import { loadScanPaths } from './scanPaths';
import { loadTags } from './tags';

/** En qué punto está la copia. */
export type BackupState =
  | { step: 'idle' }
  | { step: 'busy' }
  | { step: 'exported'; message: string }
  | { step: 'preview'; path: string; plan: ImportPlan }
  | { step: 'imported'; plan: ImportPlan }
  | { step: 'error'; message: string };

/** Estado de la sección de copia. */
export const backupState = writable<BackupState>({ step: 'idle' });

/** Exporta a `path`. */
export async function exportTo(path: string, includeWork: boolean): Promise<void> {
  backupState.set({ step: 'busy' });
  try {
    const summary = await exportBackup(path, includeWork);
    backupState.set({ step: 'exported', message: describeExport(summary, includeWork) });
  } catch (error) {
    backupState.set({ step: 'error', message: String(error) });
  }
}

/** Lee la copia de `path` y enseña lo que haría importarla. */
export async function previewFrom(path: string): Promise<void> {
  backupState.set({ step: 'busy' });
  try {
    backupState.set({ step: 'preview', path, plan: await previewImport(path) });
  } catch (error) {
    backupState.set({ step: 'error', message: String(error) });
  }
}

/** Importa la copia previsualizada y recarga todo lo que puede haber cambiado. */
export async function confirmImport(path: string): Promise<void> {
  backupState.set({ step: 'busy' });
  try {
    const plan = await applyImport(path);
    await Promise.all([
      loadScanPaths(),
      loadTags(),
      loadProjects(),
      loadApps(),
      loadAdvancedSettings(),
    ]);
    backupState.set({ step: 'imported', plan });
  } catch (error) {
    backupState.set({ step: 'error', message: String(error) });
  }
}

/** Descarta el resumen o el mensaje. */
export function dismissBackup(): void {
  backupState.set({ step: 'idle' });
}
