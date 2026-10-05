<script lang="ts">
  import { FolderSearch, Info, LayoutGrid, Rows3 } from '@lucide/svelte';

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
    density,
    detailTipDismissed,
    dismissDetailTip,
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
   * El aviso no lleva botón propio: «Limpiar filtros» ya está en la barra de
   * justo encima siempre que hay algo filtrado, y dos iguales a la vez era
   * ruido.
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
      <p class="alerta" role="alert">{error}</p>
    {/each}

    {#if hasProjects}
      <div class="herramientas">
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
            class="densidad"
          >
            {#if $density === 'comodo'}
              <Rows3 size={14} strokeWidth={1.75} />
            {:else}
              <LayoutGrid size={14} strokeWidth={1.75} />
            {/if}
          </button>
        </div>
        <FilterBar />

        {#if !$detailTipDismissed}
          <!-- Abrir el detalle no se descubre solo: el usuario lo buscó con doble clic. -->
          <p class="pista" role="note">
            <Info size={14} strokeWidth={1.75} />
            <span>
              Pulsa el nombre de un proyecto para ver su README, sus commits, sus ramas y tus notas.
            </span>
            <button type="button" onclick={dismissDetailTip}>Entendido</button>
          </p>
        {/if}

        {#if muestraAviso}
          <p class="aviso-pocos">
            {visibles === 1
              ? '1 proyecto coincide con los filtros aplicados.'
              : `${visibles} proyectos coinciden con los filtros aplicados.`}
          </p>
        {/if}
      </div>
    {/if}

    {#if $loadingProjects && !hasProjects}
      <p class="cargando">Cargando proyectos…</p>
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

  .alerta {
    margin: var(--space-4) var(--space-5) 0;
    padding: var(--space-3) var(--space-4);
    border: 1px solid color-mix(in oklab, var(--danger) 40%, transparent);
    border-radius: var(--radius-md);
    background: var(--danger-surface);
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    color: var(--danger);
  }

  .herramientas {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-5);
    border-bottom: 1px solid var(--border-subtle);
  }

  .cargando {
    margin: 0;
    padding: var(--space-8) var(--space-5);
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    text-align: center;
    color: var(--text-tertiary);
  }

  /* Mismo botón de icono que la dirección de orden, que tiene al lado. */
  .densidad {
    display: inline-flex;
    flex: none;
    padding: 6px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    transition:
      background var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .densidad:hover {
    background: var(--surface-sunken);
    color: var(--text-primary);
  }
  .densidad:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .pista {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin: 0;
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--accent-border);
    border-radius: var(--radius-md);
    background: var(--accent-surface);
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--text-primary);
  }
  .pista :global(svg) {
    flex: none;
    color: var(--accent);
  }
  .pista span {
    flex: 1;
  }
  .pista button {
    flex: none;
    padding: 3px var(--space-2);
    border: 1px solid var(--accent-border);
    border-radius: var(--radius-sm);
    background: transparent;
    font: inherit;
    color: var(--accent);
    cursor: pointer;
    transition: background var(--duration-fast) var(--ease-out);
  }
  .pista button:hover {
    background: var(--accent-surface);
  }
  .pista button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
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
</style>
