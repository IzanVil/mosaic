<script lang="ts">
  import { FolderSearch } from '@lucide/svelte';

  import EmptyState from '../components/EmptyState.svelte';
  import ProjectGrid from '../components/ProjectGrid.svelte';
  import ScanSummaryBar from '../components/ScanSummaryBar.svelte';
  import { appsError } from '../stores/apps';
  import { lastScan, loadingProjects, projects, projectsError, scanning } from '../stores/projects';
  import { scanPaths } from '../stores/scanPaths';

  interface Props {
    /** Lleva al usuario a la vista de ajustes para configurar rutas. */
    onGoToSettings: () => void;
    onScan: () => void;
  }

  let { onGoToSettings, onScan }: Props = $props();

  let hasScanPaths = $derived($scanPaths.length > 0);
</script>

<section class="flex flex-1 flex-col">
  {#if $lastScan}
    <ScanSummaryBar summary={$lastScan} />
  {/if}

  {#if $appsError}
    <p
      class="m-4 rounded-md border border-surface-border bg-surface-1 px-4 py-3 text-sm text-content"
      role="alert"
    >
      {$appsError}
    </p>
  {/if}

  {#if $projectsError}
    <p
      class="m-4 rounded-md border border-surface-border bg-surface-1 px-4 py-3 text-sm text-content"
      role="alert"
    >
      {$projectsError}
    </p>
  {/if}

  {#if $loadingProjects && $projects.length === 0}
    <p class="px-6 py-16 text-center text-sm text-content-muted">Cargando proyectos…</p>
  {:else if $projects.length === 0 && !hasScanPaths}
    <EmptyState
      title="Todavía no hay rutas que escanear"
      description="Añade la carpeta donde guardas tus proyectos y Mosaic los encontrará por ti."
      actionLabel="Configurar rutas"
      onAction={onGoToSettings}
    >
      {#snippet icon()}
        <FolderSearch size={32} strokeWidth={1.5} />
      {/snippet}
    </EmptyState>
  {:else if $projects.length === 0}
    <EmptyState
      title="Ningún proyecto encontrado"
      description="Las rutas están configuradas pero aún no se ha escaneado, o no hay proyectos dentro."
      actionLabel={$scanning ? 'Escaneando…' : 'Escanear ahora'}
      onAction={onScan}
    >
      {#snippet icon()}
        <FolderSearch size={32} strokeWidth={1.5} />
      {/snippet}
    </EmptyState>
  {:else}
    <ProjectGrid projects={$projects} />
  {/if}
</section>
