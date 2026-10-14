/** Textos de la copia de seguridad. */

import { get } from 'svelte/store';

import { lang, translate, type Key, type Lang } from '../i18n';
import type { ExportSummary, ImportPlan } from '../types';

/** Línea del resumen de una importación. */
export interface PlanLine {
  text: string;
  tone: 'change' | 'note' | 'warning';
}

const SETTING_LABELS: Record<string, Extract<Key, `copia.ajuste.${string}`>> = {
  max_depth: 'copia.ajuste.max_depth',
  excluded_dirs: 'copia.ajuste.excluded_dirs',
  max_entries_per_scan: 'copia.ajuste.max_entries_per_scan',
  scan_on_startup: 'copia.ajuste.scan_on_startup',
  git_refresh_interval_minutes: 'copia.ajuste.git_refresh_interval_minutes',
  preferred_ide: 'copia.ajuste.preferred_ide',
  preferred_terminal: 'copia.ajuste.preferred_terminal',
};

/** Nombre propuesto para el fichero de copia. */
export function backupFileName(date: Date, language: Lang = get(lang)): string {
  const pad = (n: number) => String(n).padStart(2, '0');
  const base = translate(language, 'copia.fichero');
  return `${base}-${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}.json`;
}

/** Resumen de una exportación en una frase. */
export function describeExport(
  summary: ExportSummary,
  includeWork: boolean,
  language: Lang = get(lang),
): string {
  const rutas = translate(language, 'copia.numRutas', { n: summary.scan_paths });
  if (!includeWork) return translate(language, 'copia.guardadaAjustes', { rutas });
  return translate(language, 'copia.guardadaTodo', {
    rutas,
    etiquetas: translate(language, 'comun.numEtiquetas', { n: summary.tags }),
    proyectos: translate(language, 'app.numProyectos', { n: summary.projects }),
  });
}

/** Lo que hará (o hizo) una importación, línea a línea. */
export function describePlan(plan: ImportPlan, language: Lang = get(lang)): PlanLine[] {
  const lines: PlanLine[] = [];
  const change = (text: string) => lines.push({ text, tone: 'change' });
  if (plan.settings_changed.length > 0) {
    const nombres = plan.settings_changed.map((key) => {
      const label = SETTING_LABELS[key];
      return label ? translate(language, label) : key;
    });
    change(translate(language, 'copia.planAjustes', { nombres: nombres.join(', ') }));
  }
  if (plan.scan_paths_added > 0) {
    change(translate(language, 'copia.planRutas', { n: plan.scan_paths_added }));
  }
  if (plan.tags_added > 0) change(translate(language, 'copia.planEtiquetas', { n: plan.tags_added }));
  if (plan.assignments_added > 0) {
    change(translate(language, 'copia.planAsignaciones', { n: plan.assignments_added }));
  }
  if (plan.pins_added > 0) change(translate(language, 'copia.planFijados', { n: plan.pins_added }));
  if (plan.notes_added > 0) change(translate(language, 'copia.planNotas', { n: plan.notes_added }));
  if (lines.length === 0) {
    lines.push({ text: translate(language, 'copia.planNada'), tone: 'note' });
  }

  if (plan.notes_conflicts.length > 0) {
    lines.push({
      tone: 'warning',
      text: translate(language, 'copia.planConflictos', {
        nombres: plan.notes_conflicts.join(', '),
        n: plan.notes_conflicts.length,
      }),
    });
  }
  if (plan.scan_paths_missing.length > 0) {
    lines.push({
      tone: 'warning',
      text: translate(language, 'copia.planRutasFaltan', {
        rutas: plan.scan_paths_missing.join(', '),
        n: plan.scan_paths_missing.length,
      }),
    });
  }
  if (plan.tags_existing > 0) {
    lines.push({
      tone: 'note',
      text: translate(language, 'copia.planEtiquetasExisten', { n: plan.tags_existing }),
    });
  }
  if (plan.projects_not_found > 0) {
    lines.push({
      tone: 'note',
      text: translate(language, 'copia.planNoEncontrados', { n: plan.projects_not_found }),
    });
  }
  if (!plan.has_work) {
    lines.push({ tone: 'note', text: translate(language, 'copia.planSinTrabajo') });
  }
  return lines;
}
