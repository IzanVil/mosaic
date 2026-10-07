/** Textos de la copia de seguridad. */

import type { ExportSummary, ImportPlan } from '../types';

/** Línea del resumen de una importación. */
export interface PlanLine {
  text: string;
  tone: 'change' | 'note' | 'warning';
}

const SETTING_LABELS: Record<string, string> = {
  max_depth: 'profundidad',
  excluded_dirs: 'carpetas excluidas',
  max_entries_per_scan: 'tope de entradas',
  scan_on_startup: 'escanear al arrancar',
  git_refresh_interval_minutes: 'refresco de Git',
  preferred_ide: 'editor',
  preferred_terminal: 'terminal',
};

function count(n: number, one: string, many: string): string {
  return `${n} ${n === 1 ? one : many}`;
}

/** Nombre propuesto para el fichero de copia. */
export function backupFileName(date: Date): string {
  const pad = (n: number) => String(n).padStart(2, '0');
  return `mosaic-copia-${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}.json`;
}

/** Resumen de una exportación en una frase. */
export function describeExport(summary: ExportSummary, includeWork: boolean): string {
  const rutas = count(summary.scan_paths, 'ruta', 'rutas');
  if (!includeWork) return `Copia guardada: ajustes y ${rutas}.`;
  return `Copia guardada: ajustes, ${rutas}, ${count(summary.tags, 'etiqueta', 'etiquetas')} y ${count(summary.projects, 'proyecto', 'proyectos')} con etiquetas, notas o fijados.`;
}

/** Lo que hará (o hizo) una importación, línea a línea. */
export function describePlan(plan: ImportPlan): PlanLine[] {
  const lines: PlanLine[] = [];
  const change = (text: string) => lines.push({ text, tone: 'change' });

  if (plan.settings_changed.length > 0) {
    const nombres = plan.settings_changed.map((key) => SETTING_LABELS[key] ?? key);
    change(`Ajustes: cambia ${nombres.join(', ')}.`);
  }
  if (plan.scan_paths_added > 0) {
    change(`Añade ${count(plan.scan_paths_added, 'ruta de escaneo', 'rutas de escaneo')}.`);
  }
  if (plan.tags_added > 0) change(`Crea ${count(plan.tags_added, 'etiqueta', 'etiquetas')}.`);
  if (plan.assignments_added > 0) {
    change(`Pone ${count(plan.assignments_added, 'etiqueta', 'etiquetas')} a tus proyectos.`);
  }
  if (plan.pins_added > 0) change(`Fija ${count(plan.pins_added, 'proyecto', 'proyectos')}.`);
  if (plan.notes_added > 0) {
    change(`Añade notas a ${count(plan.notes_added, 'proyecto', 'proyectos')}.`);
  }

  if (lines.length === 0) {
    lines.push({ text: 'No cambia nada: ya tienes todo lo que trae esta copia.', tone: 'note' });
  }

  if (plan.notes_conflicts.length > 0) {
    lines.push({
      tone: 'warning',
      text: `${plan.notes_conflicts.join(', ')}: ya ${plan.notes_conflicts.length === 1 ? 'tiene' : 'tienen'} notas distintas. Se quedan las tuyas.`,
    });
  }
  if (plan.scan_paths_missing.length > 0) {
    lines.push({
      tone: 'warning',
      text: `${count(plan.scan_paths_missing.length, 'ruta no existe', 'rutas no existen')} en este equipo y se ${plan.scan_paths_missing.length === 1 ? 'salta' : 'saltan'}: ${plan.scan_paths_missing.join(', ')}.`,
    });
  }
  if (plan.tags_existing > 0) {
    lines.push({
      tone: 'note',
      text: `${count(plan.tags_existing, 'etiqueta ya existía', 'etiquetas ya existían')} y ${plan.tags_existing === 1 ? 'conserva su' : 'conservan su'} color.`,
    });
  }
  if (plan.projects_not_found > 0) {
    lines.push({
      tone: 'note',
      text: `${count(plan.projects_not_found, 'proyecto de la copia no está', 'proyectos de la copia no están')} en este equipo. Escanea y vuelve a importar para completarlos.`,
    });
  }
  if (!plan.has_work) {
    lines.push({ tone: 'note', text: 'La copia no trae etiquetas ni notas.' });
  }
  return lines;
}
