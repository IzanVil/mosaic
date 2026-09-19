<script lang="ts">
  import { Pin, SearchX } from '@lucide/svelte';

  import EmptyState from './EmptyState.svelte';
  import ProjectCard from './ProjectCard.svelte';
  import { clearFilters } from '../stores/filters';
  import type { ProjectWithTags } from '../types';

  interface Props {
    /** Proyectos destacados, que se pintan siempre en el bloque de arriba. */
    pinned: ProjectWithTags[];
    /** El resto, ya filtrado y ordenado. */
    rest: ProjectWithTags[];
  }

  let { pinned, rest }: Props = $props();

  const COLUMNAS = 'repeat(auto-fill, minmax(280px, 1fr))';

  let total = $derived(pinned.length + rest.length);
</script>

{#if total === 0}
  <EmptyState
    title="Ningún proyecto coincide con los filtros"
    description="Prueba con otra búsqueda, o quita los filtros para volver a ver todo."
    actionLabel="Limpiar filtros"
    onAction={clearFilters}
  >
    {#snippet icon()}
      <SearchX size={32} strokeWidth={1.5} />
    {/snippet}
  </EmptyState>
{:else}
  <div class="flex flex-col gap-5 p-6">
    {#if pinned.length > 0}
      <section class="flex flex-col gap-3" aria-label="Proyectos fijados">
        <h2 class="flex items-center gap-1.5 text-xs font-medium uppercase tracking-wide
                   text-content-muted">
          <Pin size={12} />
          Fijados
        </h2>
        <div class="grid gap-3" style="grid-template-columns: {COLUMNAS};">
          {#each pinned as project (project.id)}
            <ProjectCard {project} />
          {/each}
        </div>
      </section>
    {/if}

    {#if pinned.length > 0 && rest.length > 0}
      <hr class="border-surface-border" />
    {/if}

    {#if rest.length > 0}
      <section class="flex flex-col gap-3" aria-label="Resto de proyectos">
        <div class="grid gap-3" style="grid-template-columns: {COLUMNAS};">
          {#each rest as project (project.id)}
            <ProjectCard {project} />
          {/each}
        </div>
      </section>
    {/if}
  </div>
{/if}
