<script lang="ts">
  import { CircleCheck, TriangleAlert } from '@lucide/svelte';

  import type { ScanSummary } from '../types';
  import { formatDuration } from '../utils/format';

  interface Props {
    summary: ScanSummary;
  }

  let { summary }: Props = $props();

  /** Detalles que solo merece la pena mostrar si no son cero. */
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

<div class="resumen" role="status">
  <p class="linea">
    <CircleCheck size={14} strokeWidth={1.75} class="icono" />
    <span>
      <span class="total">
        {summary.projects_found}
        {summary.projects_found === 1 ? 'proyecto encontrado' : 'proyectos encontrados'}
      </span>
      {#if details.length > 0}
        <span class="detalle">· {details.join(' · ')}</span>
      {/if}
      <span class="detalle">· en {formatDuration(summary.elapsed_ms)}</span>
    </span>
  </p>

  {#each warnings as warning (warning)}
    <p class="aviso">
      <TriangleAlert size={14} strokeWidth={1.75} class="icono" />
      {warning}
    </p>
  {/each}
</div>

<style>
  .resumen {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding: 10px var(--space-5);
    border-bottom: 1px solid var(--border-subtle);
    background: var(--surface-raised);
  }

  .linea,
  .aviso {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin: 0;
  }

  .linea {
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    color: var(--text-secondary);
  }

  .linea :global(.icono) {
    flex: none;
    color: var(--success);
  }

  .total {
    font-weight: 500;
    color: var(--text-primary);
  }

  .detalle {
    color: var(--text-tertiary);
  }

  .aviso {
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--warning);
  }

  .aviso :global(.icono) {
    flex: none;
  }
</style>
