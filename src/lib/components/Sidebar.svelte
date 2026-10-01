<script lang="ts">
  import {
    LayoutGrid,
    PanelLeftClose,
    PanelLeftOpen,
    Pin,
    Settings2,
    Tags,
  } from '@lucide/svelte';

  import { clearFilters, filters, setPinnedOnly, toggleTagFilter } from '../stores/filters';
  import { pinnedProjects, projects } from '../stores/projects';
  import { tags } from '../stores/tags';

  interface Props {
    collapsed: boolean;
    onToggle: () => void;
    onManageTags: () => void;
  }

  let { collapsed, onToggle, onManageTags }: Props = $props();

  let showingAll = $derived(
    $filters.tag_ids.length === 0 && !$filters.pinned_only && $filters.languages.length === 0,
  );

  function showAll() {
    clearFilters();
  }
</script>

{#if collapsed}
  <nav class="panel plegado" aria-label="Colecciones">
    <button
      type="button"
      class="icono"
      onclick={onToggle}
      aria-label="Desplegar el panel lateral"
      title="Desplegar el panel lateral"
    >
      <PanelLeftOpen size={16} strokeWidth={1.75} />
    </button>

    <button
      type="button"
      class="icono"
      class:activo={showingAll}
      onclick={showAll}
      aria-label="Todos los proyectos"
      title="Todos los proyectos"
    >
      <LayoutGrid size={16} strokeWidth={1.75} />
    </button>

    {#if $pinnedProjects.length > 0}
      <button
        type="button"
        class="icono"
        class:activo={$filters.pinned_only}
        onclick={() => setPinnedOnly(!$filters.pinned_only)}
        aria-label="Solo destacados"
        title="Solo destacados"
      >
        <Pin size={16} strokeWidth={1.75} />
      </button>
    {/if}

    <div class="puntos">
      {#each $tags as tag (tag.id)}
        <button
          type="button"
          class="punto"
          class:elegido={$filters.tag_ids.includes(tag.id)}
          onclick={() => toggleTagFilter(tag.id)}
          aria-label="Filtrar por {tag.name}"
          aria-pressed={$filters.tag_ids.includes(tag.id)}
          title="{tag.name} ({tag.project_count})"
          style="background-color: {tag.color};"
        ></button>
      {/each}
    </div>

    <button
      type="button"
      class="icono final"
      onclick={onManageTags}
      aria-label="Gestionar etiquetas"
      title="Gestionar etiquetas"
    >
      <Settings2 size={16} strokeWidth={1.75} />
    </button>
  </nav>
{:else}
  <nav class="panel" aria-label="Colecciones">
    <div class="cabecera">
      <span class="rotulo">Vistas</span>
      <button
        type="button"
        class="icono pequeno"
        onclick={onToggle}
        aria-label="Plegar el panel lateral"
        title="Plegar el panel lateral"
      >
        <PanelLeftClose size={14} strokeWidth={1.75} />
      </button>
    </div>

    <button type="button" class="fila" class:activo={showingAll} onclick={showAll}>
      <LayoutGrid size={14} strokeWidth={1.75} class="glifo" />
      <span class="nombre">Todos los proyectos</span>
      <span class="numero">{$projects.length}</span>
    </button>

    {#if $pinnedProjects.length > 0}
      <button
        type="button"
        class="fila"
        class:activo={$filters.pinned_only}
        onclick={() => setPinnedOnly(!$filters.pinned_only)}
        aria-pressed={$filters.pinned_only}
      >
        <Pin size={14} strokeWidth={1.75} class="glifo" />
        <span class="nombre">Fijados</span>
        <span class="numero">{$pinnedProjects.length}</span>
      </button>
    {/if}

    <div class="cabecera separada">
      <span class="rotulo">Etiquetas</span>
      <button
        type="button"
        class="icono pequeno"
        onclick={onManageTags}
        aria-label="Gestionar etiquetas"
        title="Gestionar etiquetas"
      >
        <Settings2 size={14} strokeWidth={1.75} />
      </button>
    </div>

    {#if $tags.length === 0}
      <p class="vacio">Ninguna todavía. Etiqueta un proyecto desde su tarjeta.</p>
    {:else}
      {#each $tags as tag (tag.id)}
        <button
          type="button"
          class="fila"
          class:activo={$filters.tag_ids.includes(tag.id)}
          onclick={() => toggleTagFilter(tag.id)}
          aria-pressed={$filters.tag_ids.includes(tag.id)}
        >
          <span class="color" style="background-color: {tag.color};"></span>
          <span class="nombre" title={tag.name}>{tag.name}</span>
          <span class="numero">{tag.project_count}</span>
        </button>
      {/each}
    {/if}

    <button type="button" class="fila gestionar" onclick={onManageTags}>
      <Tags size={14} strokeWidth={1.75} class="glifo" />
      <span class="nombre">Gestionar etiquetas</span>
    </button>
  </nav>
{/if}

<style>
  .panel {
    display: flex;
    flex: none;
    flex-direction: column;
    gap: 2px;
    width: 14rem;
    overflow-y: auto;
    padding: var(--space-3) var(--space-2);
    border-right: 1px solid var(--border-subtle);
    background: var(--surface-raised);
  }

  .panel.plegado {
    width: 3rem;
    align-items: center;
    gap: var(--space-2);
    padding-inline: 0;
  }

  .cabecera {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-1);
    padding: 0 var(--space-2) var(--space-1);
  }
  .cabecera.separada {
    margin-top: var(--space-4);
  }

  .rotulo {
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    font-weight: 500;
    letter-spacing: var(--tracking-wide);
    text-transform: uppercase;
    color: var(--text-tertiary);
  }

  .fila {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 6px var(--space-2);
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    font: inherit;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    text-align: left;
    color: var(--text-secondary);
    cursor: pointer;
    transition:
      background var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .fila:hover {
    background: var(--surface-sunken);
    color: var(--text-primary);
  }
  .fila.activo {
    background: var(--surface-sunken);
    color: var(--text-primary);
  }
  .fila :global(.glifo) {
    flex: none;
    color: var(--text-tertiary);
  }

  .nombre {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .numero {
    flex: none;
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    font-variant-numeric: tabular-nums;
    color: var(--text-tertiary);
  }

  .color {
    flex: none;
    width: 10px;
    height: 10px;
    border-radius: var(--radius-pill);
  }

  .gestionar {
    margin-top: auto;
    color: var(--text-tertiary);
  }

  .vacio {
    margin: 0;
    padding: var(--space-1) var(--space-2);
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--text-tertiary);
  }

  .icono {
    display: inline-flex;
    padding: var(--space-2);
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    transition:
      background var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .icono.pequeno {
    padding: var(--space-1);
  }
  .icono:hover {
    background: var(--surface-sunken);
    color: var(--text-primary);
  }
  .icono.activo {
    color: var(--accent);
  }
  .icono.final {
    margin-top: auto;
  }

  .puntos {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    margin-top: var(--space-1);
    overflow-y: auto;
  }

  .punto {
    width: 14px;
    height: 14px;
    padding: 0;
    border: 2px solid transparent;
    border-radius: var(--radius-pill);
    cursor: pointer;
    transition: transform var(--duration-fast) var(--ease-out);
  }
  .punto:hover {
    transform: scale(1.1);
  }
  .punto.elegido {
    border-color: var(--text-primary);
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
