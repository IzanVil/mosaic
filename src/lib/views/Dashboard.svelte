<script lang="ts">
  import { FolderSearch, LayoutGrid, Rows3 } from '@lucide/svelte';

  import EmptyState from '../components/EmptyState.svelte';
  import FilterBar from '../components/FilterBar.svelte';
  import ProjectGrid from '../components/ProjectGrid.svelte';
  import ScanSummaryBar from '../components/ScanSummaryBar.svelte';
  import SearchBar from '../components/SearchBar.svelte';
  import Sidebar from '../components/Sidebar.svelte';
  import SortMenu from '../components/SortMenu.svelte';
  import TagManager from '../components/TagManager.svelte';
  import { appsError } from '../stores/apps';
  import {
    clearFilters,
    density,
    hasActiveFilters,
    sidebarCollapsed,
    toggleDensity,
    toggleSidebar,
  } from '../stores/filters';
  import {
    lastScan,
    loadingProjects,
    projects,
    projectsError,
    scanning,
    visibleGroups,
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

  /**
   * Umbral a partir del cual se explica por qué hay tan poco en pantalla.
   *
   * La rejilla NO se recoloca: la búsqueda filtra en vivo, y centrar el
   * contenido al bajar de seis resultados haría saltar las tarjetas a mitad de
   * pantalla entre una letra y la siguiente.
   */
  const POCOS_RESULTADOS = 5;

  let visibles = $derived($visibleProjects.length);
  let muestraAviso = $derived($hasActiveFilters && visibles > 0 && visibles <= POCOS_RESULTADOS);
</script>

<section class="flex min-h-0 flex-1">
  {#if hasProjects}
    <Sidebar
      collapsed={$sidebarCollapsed}
      onToggle={toggleSidebar}
      onManageTags={() => (managingTags = true)}
    />
  {/if}

  <div class="lienzo flex min-w-0 flex-1 flex-col overflow-y-auto">
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
          <button
            type="button"
            onclick={toggleDensity}
            title={$density === 'comodo'
              ? 'Cambiar a vista compacta'
              : 'Cambiar a vista cómoda'}
            aria-label={$density === 'comodo'
              ? 'Cambiar a vista compacta'
              : 'Cambiar a vista cómoda'}
            aria-pressed={$density === 'compacto'}
            class="rounded-md border border-surface-border p-1.5 text-content-muted transition
                   hover:bg-surface-2 hover:text-content focus-visible:outline-2
                   focus-visible:outline-offset-2 focus-visible:outline-accent"
          >
            {#if $density === 'comodo'}
              <Rows3 size={14} strokeWidth={1.75} />
            {:else}
              <LayoutGrid size={14} strokeWidth={1.75} />
            {/if}
          </button>
        </div>
        <FilterBar />

        {#if muestraAviso}
          <p class="aviso-pocos">
            {visibles === 1
              ? '1 proyecto coincide con los filtros aplicados.'
              : `${visibles} proyectos coinciden con los filtros aplicados.`}
            <button type="button" onclick={clearFilters}>Limpiar filtros</button>
          </p>
        {/if}
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
      <ProjectGrid
        pinned={$visibleGroups.pinned}
        rest={$visibleGroups.rest}
        density={$density}
      />
    {/if}
  </div>
</section>

{#if managingTags}
  <TagManager onClose={() => (managingTags = false)} />
{/if}

<style>
  /*
   * El fondo de la vista pasa al token nuevo, cinco puntos de luminosidad por
   * debajo de la superficie de la tarjeta. Con el token anterior la diferencia
   * era de uno y las tarjetas no se despegaban del fondo.
   */
  .lienzo {
    background: var(--surface-base);
  }

  .aviso-pocos {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin: 0;
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--text-tertiary);
  }

  .aviso-pocos button {
    border: 0;
    background: none;
    padding: 0;
    font: inherit;
    color: var(--accent);
    cursor: pointer;
    text-decoration: underline;
    text-underline-offset: 2px;
  }
  .aviso-pocos button:hover {
    color: var(--accent-hover);
  }
  .aviso-pocos button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
    border-radius: var(--radius-sm);
  }
</style>
