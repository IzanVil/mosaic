<script lang="ts">
  import { FolderSearch } from '@lucide/svelte';

  import EmptyState from '../components/EmptyState.svelte';
  import FilterBar from '../components/FilterBar.svelte';
  import ProjectGrid from '../components/ProjectGrid.svelte';
  import ScanSummaryBar from '../components/ScanSummaryBar.svelte';
  import SearchBar from '../components/SearchBar.svelte';
  import Sidebar from '../components/Sidebar.svelte';
  import SortMenu from '../components/SortMenu.svelte';
  import TagManager from '../components/TagManager.svelte';
  import { appsError } from '../stores/apps';
  import { sidebarCollapsed, toggleSidebar } from '../stores/filters';
  import {
    lastScan,
    loadingProjects,
    projects,
    projectsError,
    scanning,
    visibleProjects,
  } from '../stores/projects';
  import { scanPaths } from '../stores/scanPaths';
  import { tagsError } from '../stores/tags';

  interface Props {
    onGoToSettings: () => void;
    onScan: () => void;
  }

  let { onGoToSettings, onScan }: Props = $props();

  let managingTags = $state(false);

  let hasScanPaths = $derived($scanPaths.length > 0);
  let hasProjects = $derived($projects.length > 0);
</script>

<section class="flex min-h-0 flex-1">
  {#if hasProjects}
    <Sidebar
      collapsed={$sidebarCollapsed}
      onToggle={toggleSidebar}
      onManageTags={() => (managingTags = true)}
    />
  {/if}

  <div class="flex min-w-0 flex-1 flex-col overflow-y-auto">
    {#if $lastScan}
      <ScanSummaryBar summary={$lastScan} />
    {/if}

    {#each [$appsError, $projectsError, $tagsError].filter((error) => error !== null) as error}
      <p
        class="mx-6 mt-4 rounded-md border border-surface-border bg-surface-1 px-4 py-3 text-sm
               text-content"
        role="alert"
      >
        {error}
      </p>
    {/each}

    {#if hasProjects}
      <div class="flex flex-col gap-2 border-b border-surface-border px-6 py-3">
        <div class="flex items-center gap-3">
          <div class="min-w-0 flex-1"><SearchBar /></div>
          <SortMenu />
        </div>
        <FilterBar />
      </div>
    {/if}

    {#if $loadingProjects && !hasProjects}
      <p class="px-6 py-16 text-center text-sm text-content-muted">Cargando proyectos…</p>
    {:else if !hasProjects && !hasScanPaths}
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
    {:else if !hasProjects}
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
      <ProjectGrid projects={$visibleProjects} />
    {/if}
  </div>
</section>

{#if managingTags}
  <TagManager onClose={() => (managingTags = false)} />
{/if}
