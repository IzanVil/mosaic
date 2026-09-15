<script lang="ts">
  import { FolderSearch, TriangleAlert } from '@lucide/svelte';

  import EmptyState from '../components/EmptyState.svelte';
  import GitStatusBadge from '../components/GitStatusBadge.svelte';
  import ScanSummaryBar from '../components/ScanSummaryBar.svelte';
  import { gitStatus } from '../stores/gitStatus';
  import { lastScan, loadingProjects, projects, projectsError, scanning } from '../stores/projects';
  import { scanPaths } from '../stores/scanPaths';
  import { formatRelativeTime, shortenPath } from '../utils/format';

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
    <ul class="divide-y divide-surface-border">
      {#each $projects as project (project.id)}
        <li class="flex items-baseline gap-3 px-6 py-3 hover:bg-surface-1">
          <span class="font-medium text-content-strong">{project.name}</span>

          {#if project.primary_language}
            <span
              class="shrink-0 rounded-full border border-surface-border px-2 py-0.5
                     text-xs text-content-muted"
            >
              {project.primary_language}
            </span>
          {/if}

          {#if project.is_git_repo}
            <GitStatusBadge status={$gitStatus[project.id]} />
          {/if}

          {#if project.missing}
            <span
              class="inline-flex shrink-0 items-center gap-1 text-xs text-content-muted"
              title="El último escaneo no encontró esta carpeta en disco"
            >
              <TriangleAlert size={12} />
              No encontrado
            </span>
          {/if}

          <span class="ml-auto shrink-0 font-mono text-xs text-content-muted" title={project.path}>
            {shortenPath(project.path)}
          </span>

          <span class="shrink-0 text-xs text-content-muted">
            {formatRelativeTime(project.last_seen_at)}
          </span>
        </li>
      {/each}
    </ul>
  {/if}
</section>
