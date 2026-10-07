/** Wrappers de los comandos de copia de seguridad. */

import { invoke } from '@tauri-apps/api/core';

import type { ExportSummary, ImportPlan } from '../types';

/** Escribe la copia en `path`. */
export function exportBackup(path: string, includeWork: boolean): Promise<ExportSummary> {
  return invoke<ExportSummary>('export_backup', { path, includeWork });
}

/** Cuenta lo que haría importar la copia, sin escribir nada. */
export function previewImport(path: string): Promise<ImportPlan> {
  return invoke<ImportPlan>('preview_import', { path });
}

/** Importa la copia. */
export function applyImport(path: string): Promise<ImportPlan> {
  return invoke<ImportPlan>('apply_import', { path });
}
