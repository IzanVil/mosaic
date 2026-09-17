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
  <nav
    class="flex w-12 shrink-0 flex-col items-center gap-2 border-r border-surface-border
           bg-surface-1 py-3"
    aria-label="Colecciones"
  >
    <button
      type="button"
      onclick={onToggle}
      aria-label="Desplegar el panel lateral"
      title="Desplegar el panel lateral"
      class="rounded-md p-2 text-content-muted transition hover:bg-surface-2 hover:text-content
             focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
    >
      <PanelLeftOpen size={16} />
    </button>

    <button
      type="button"
      onclick={showAll}
      aria-label="Todos los proyectos"
      title="Todos los proyectos"
      class="rounded-md p-2 transition hover:bg-surface-2 focus-visible:outline-2
             focus-visible:outline-offset-2 focus-visible:outline-accent"
      class:text-accent={showingAll}
      class:text-content-muted={!showingAll}
    >
      <LayoutGrid size={16} />
    </button>

    {#if $pinnedProjects.length > 0}
      <button
        type="button"
        onclick={() => setPinnedOnly(!$filters.pinned_only)}
        aria-label="Solo destacados"
        title="Solo destacados"
        class="rounded-md p-2 transition hover:bg-surface-2 focus-visible:outline-2
               focus-visible:outline-offset-2 focus-visible:outline-accent"
        class:text-accent={$filters.pinned_only}
        class:text-content-muted={!$filters.pinned_only}
      >
        <Pin size={16} />
      </button>
    {/if}

    <div class="mt-1 flex flex-col items-center gap-1.5 overflow-y-auto">
      {#each $tags as tag (tag.id)}
        <button
          type="button"
          onclick={() => toggleTagFilter(tag.id)}
          aria-label="Filtrar por {tag.name}"
          aria-pressed={$filters.tag_ids.includes(tag.id)}
          title="{tag.name} ({tag.project_count})"
          class="size-3.5 rounded-full border-2 transition hover:scale-110 focus-visible:outline-2
                 focus-visible:outline-offset-1 focus-visible:outline-accent"
          style="background-color: {tag.color}; border-color: {$filters.tag_ids.includes(tag.id)
            ? 'currentColor'
            : 'transparent'};"
        ></button>
      {/each}
    </div>

    <button
      type="button"
      onclick={onManageTags}
      aria-label="Gestionar etiquetas"
      title="Gestionar etiquetas"
      class="mt-auto rounded-md p-2 text-content-muted transition hover:bg-surface-2
             hover:text-content focus-visible:outline-2 focus-visible:outline-offset-2
             focus-visible:outline-accent"
    >
      <Settings2 size={16} />
    </button>
  </nav>
{:else}
  <nav
    class="flex w-56 shrink-0 flex-col gap-1 overflow-y-auto border-r border-surface-border
           bg-surface-1 px-2 py-3"
    aria-label="Colecciones"
  >
    <div class="flex items-center justify-between gap-1 px-2 pb-1">
      <span class="text-xs font-medium uppercase tracking-wide text-content-muted">Vistas</span>
      <button
        type="button"
        onclick={onToggle}
        aria-label="Plegar el panel lateral"
        title="Plegar el panel lateral"
        class="rounded p-1 text-content-muted transition hover:bg-surface-2 hover:text-content
               focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
      >
        <PanelLeftClose size={15} />
      </button>
    </div>

    <button
      type="button"
      onclick={showAll}
      class="flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm transition
             hover:bg-surface-2 focus-visible:outline-2 focus-visible:outline-offset-2
             focus-visible:outline-accent"
      class:bg-surface-2={showingAll}
      class:text-content-strong={showingAll}
      class:text-content={!showingAll}
    >
      <LayoutGrid size={15} class="shrink-0 text-content-muted" />
      <span class="min-w-0 flex-1 truncate">Todos los proyectos</span>
      <span class="shrink-0 text-xs text-content-muted">{$projects.length}</span>
    </button>

    {#if $pinnedProjects.length > 0}
      <button
        type="button"
        onclick={() => setPinnedOnly(!$filters.pinned_only)}
        aria-pressed={$filters.pinned_only}
        class="flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm transition
               hover:bg-surface-2 focus-visible:outline-2 focus-visible:outline-offset-2
               focus-visible:outline-accent"
        class:bg-surface-2={$filters.pinned_only}
        class:text-content-strong={$filters.pinned_only}
        class:text-content={!$filters.pinned_only}
      >
        <Pin size={15} class="shrink-0 text-content-muted" />
        <span class="min-w-0 flex-1 truncate">Fijados</span>
        <span class="shrink-0 text-xs text-content-muted">{$pinnedProjects.length}</span>
      </button>
    {/if}

    <div class="mt-3 flex items-center justify-between gap-1 px-2 pb-1">
      <span class="text-xs font-medium uppercase tracking-wide text-content-muted">Etiquetas</span>
      <button
        type="button"
        onclick={onManageTags}
        aria-label="Gestionar etiquetas"
        title="Gestionar etiquetas"
        class="rounded p-1 text-content-muted transition hover:bg-surface-2 hover:text-content
               focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
      >
        <Settings2 size={15} />
      </button>
    </div>

    {#if $tags.length === 0}
      <p class="px-2 py-1 text-xs text-content-muted">
        Ninguna todavía. Etiqueta un proyecto desde su tarjeta.
      </p>
    {:else}
      {#each $tags as tag (tag.id)}
        <button
          type="button"
          onclick={() => toggleTagFilter(tag.id)}
          aria-pressed={$filters.tag_ids.includes(tag.id)}
          class="flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm transition
                 hover:bg-surface-2 focus-visible:outline-2 focus-visible:outline-offset-2
                 focus-visible:outline-accent"
          class:bg-surface-2={$filters.tag_ids.includes(tag.id)}
          class:text-content-strong={$filters.tag_ids.includes(tag.id)}
          class:text-content={!$filters.tag_ids.includes(tag.id)}
        >
          <span
            class="size-2.5 shrink-0 rounded-full"
            style="background-color: {tag.color};"
          ></span>
          <span class="min-w-0 flex-1 truncate" title={tag.name}>{tag.name}</span>
          <span class="shrink-0 text-xs text-content-muted">{tag.project_count}</span>
        </button>
      {/each}
    {/if}

    <button
      type="button"
      onclick={onManageTags}
      class="mt-auto flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm
             text-content-muted transition hover:bg-surface-2 hover:text-content
             focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
    >
      <Tags size={15} />
      Gestionar etiquetas
    </button>
  </nav>
{/if}
