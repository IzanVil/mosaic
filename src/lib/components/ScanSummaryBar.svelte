<script lang="ts">
  import { CircleCheck, TriangleAlert } from '@lucide/svelte';

  import type { ScanSummary } from '../types';
  import { formatDuration } from '../utils/format';

  interface Props {
    summary: ScanSummary;
  }

  let { summary }: Props = $props();

  let details = $derived(
    [
      summary.projects_new > 0 ? `${summary.projects_new} nuevos` : null,
      summary.projects_updated > 0 ? `${summary.projects_updated} actualizados` : null,
      summary.projects_missing > 0 ? `${summary.projects_missing} ya no están` : null,
    ].filter((detail) => detail !== null),
  );

  let warnings = $derived(
    [
      summary.truncated
        ? 'El escaneo se detuvo al alcanzar el tope de entradas, así que puede faltar algo. Súbelo en los ajustes o acota las rutas.'
        : null,
      summary.roots_unavailable > 0
        ? `${summary.roots_unavailable} ${summary.roots_unavailable === 1 ? 'ruta no existe' : 'rutas no existen'} en el disco.`
        : null,
      summary.roots_scanned === 0 && summary.roots_unavailable === 0
        ? 'No hay ninguna ruta habilitada que escanear.'
        : null,
    ].filter((warning) => warning !== null),
  );
</script>

<div
  class="flex flex-col gap-1 border-b border-surface-border bg-surface-1 px-6 py-2.5 text-sm"
  role="status"
>
  <p class="flex items-center gap-2 text-content">
    <CircleCheck size={14} class="shrink-0 text-content-muted" />
    <span>
      <span class="font-medium text-content-strong">
        {summary.projects_found}
        {summary.projects_found === 1 ? 'proyecto encontrado' : 'proyectos encontrados'}
      </span>
      {#if details.length > 0}
        <span class="text-content-muted">· {details.join(' · ')}</span>
      {/if}
      <span class="text-content-muted">· en {formatDuration(summary.elapsed_ms)}</span>
    </span>
  </p>

  {#each warnings as warning (warning)}
    <p class="flex items-center gap-2 text-xs text-content-muted">
      <TriangleAlert size={13} class="shrink-0" />
      {warning}
    </p>
  {/each}
</div>
