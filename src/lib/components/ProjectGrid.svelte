<script lang="ts">
  import { SearchX } from '@lucide/svelte';

  import EmptyState from './EmptyState.svelte';
  import ProjectCard from './ProjectCard.svelte';
  import { clearFilters } from '../stores/filters';
  import type { ProjectWithTags } from '../types';

  interface Props {
    projects: ProjectWithTags[];
  }

  let { projects }: Props = $props();
</script>

{#if projects.length === 0}
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
  <div class="grid gap-3 p-6" style="grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));">
    {#each projects as project (project.id)}
      <ProjectCard {project} />
    {/each}
  </div>
{/if}
